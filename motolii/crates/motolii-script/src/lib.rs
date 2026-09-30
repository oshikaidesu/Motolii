//! Script execution knows only commands and read-only queries, never the editor or GPU.
use rquickjs::{CatchResultExt, CaughtError, Context, Ctx, Exception, Function, Runtime};
use serde_json::Value;
use std::time::{Duration, Instant};

const PRELUDE: &str = include_str!("prelude.js");
pub const DEFAULT_BUDGET: Duration = Duration::from_secs(20);

#[derive(Clone, Copy, Debug)]
pub enum Query {
    Layer(u64),
    Effects,
    Assets,
    Composition,
}

/// The host owns validation, Document/Undo and any side effects of a command.
pub trait Host {
    fn command(&mut self, request: Value) -> Result<Value, String>;
    fn query(&self, query: Query) -> Result<Value, String>;
}

pub fn execute<H: Host + 'static>(
    host: &mut H,
    source: &str,
    name: &str,
    budget: Duration,
) -> Result<(), String> {
    // QuickJS callbacks require 'static captures. This pointer cannot escape: the
    // context/runtime are local, synchronous and dropped before the exclusive
    // host borrow ends. No JS handle or callback is returned to the caller.
    let host: *mut H = host;
    let runtime = Runtime::new().map_err(|e| e.to_string())?;
    runtime.set_memory_limit(512 * 1024 * 1024);
    let started = Instant::now();
    runtime.set_interrupt_handler(Some(Box::new(move || started.elapsed() > budget)));
    let context = Context::full(&runtime).map_err(|e| e.to_string())?;
    let outcome = context.with(|ctx| -> Result<(), String> {
        let bind = || -> rquickjs::Result<()> {
            let globals = ctx.globals();
            globals.set(
                "__op",
                Function::new(
                    ctx.clone(),
                    move |ctx: Ctx<'_>, request: String| -> rquickjs::Result<String> {
                        let request = serde_json::from_str(&request)
                            .map_err(|e| Exception::throw_message(&ctx, &e.to_string()))?;
                        unsafe { &mut *host }
                            .command(request)
                            .map(|v| v.to_string())
                            .map_err(|e| Exception::throw_message(&ctx, &e))
                    },
                )?,
            )?;
            globals.set(
                "__layer",
                Function::new(
                    ctx.clone(),
                    move |ctx: Ctx<'_>, id: f64| -> rquickjs::Result<String> {
                        unsafe { &*host }
                            .query(Query::Layer(id as u64))
                            .map(|v| v.to_string())
                            .map_err(|e| Exception::throw_message(&ctx, &e))
                    },
                )?,
            )?;
            for (name, query) in [
                ("__effects", Query::Effects),
                ("__assets", Query::Assets),
                ("__comp", Query::Composition),
            ] {
                globals.set(
                    name,
                    Function::new(
                        ctx.clone(),
                        move |ctx: Ctx<'_>| -> rquickjs::Result<String> {
                            unsafe { &*host }
                                .query(query)
                                .map(|v| v.to_string())
                                .map_err(|e| Exception::throw_message(&ctx, &e))
                        },
                    )?,
                )?;
            }
            Ok(())
        };
        bind().map_err(|e| e.to_string())?;
        let run = |source: &str, file: &str| {
            let mut options = rquickjs::context::EvalOptions::default();
            options.filename = Some(file.to_owned());
            ctx.eval_with_options::<(), _>(source, options)
                .catch(&ctx)
                .map_err(|e| match e {
                    CaughtError::Exception(e) => format!(
                        "{}{}",
                        e.message().unwrap_or_default(),
                        e.stack().map(|s| format!("\n{s}")).unwrap_or_default()
                    ),
                    other => other.to_string(),
                })
        };
        run(PRELUDE, "motolii-prelude.js")?;
        run(source, name)
    });
    if started.elapsed() > budget {
        return Err(format!(
            "{name}: stopped after {:.1} seconds (a loop that never ends?)",
            budget.as_secs_f64()
        ));
    }
    outcome
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[derive(Default)]
    struct RecordingHost {
        commands: Vec<Value>,
    }
    impl Host for RecordingHost {
        fn command(&mut self, request: Value) -> Result<Value, String> {
            if request["op"] == "reject" {
                return Err("host refused".into());
            }
            self.commands.push(request);
            Ok(json!({"selected": 7}))
        }
        fn query(&self, query: Query) -> Result<Value, String> {
            Ok(match query {
                Query::Layer(id) => json!({"id": id}),
                Query::Effects | Query::Assets => json!([]),
                Query::Composition => {
                    json!({"fps": 30, "width": 1920, "height": 1080, "seconds": 4})
                }
            })
        }
    }

    #[test]
    fn prelude_routes_commands_without_an_editor_or_renderer() {
        let mut host = RecordingHost::default();
        execute(&mut host, "rectangle();", "example.js", DEFAULT_BUDGET).unwrap();
        assert!(host
            .commands
            .iter()
            .any(|r| r["op"] == "create" && r["kind"] == "rectangle"));
    }

    #[test]
    fn queries_are_read_only_and_keep_wire_values() {
        let mut host = RecordingHost::default();
        execute(
            &mut host,
            r#"
            if (JSON.parse(__layer(42)).id !== 42) throw Error('layer');
            if (JSON.parse(__comp()).fps !== 30) throw Error('comp');
            if (__assets() !== '[]' || __effects() !== '[]') throw Error('catalog');
        "#,
            "queries.js",
            DEFAULT_BUDGET,
        )
        .unwrap();
        assert!(host.commands.is_empty());
    }

    #[test]
    fn host_refusal_keeps_source_location() {
        let error = execute(
            &mut RecordingHost::default(),
            "op('reject');",
            "refused.js",
            DEFAULT_BUDGET,
        )
        .unwrap_err();
        assert!(
            error.contains("host refused") && error.contains("refused.js"),
            "{error}"
        );
    }

    #[test]
    fn infinite_loop_is_interrupted_without_a_gpu() {
        let error = execute(
            &mut RecordingHost::default(),
            "for (;;) {}",
            "loop.js",
            Duration::from_millis(50),
        )
        .unwrap_err();
        assert!(error.contains("stopped"), "{error}");
    }
}
