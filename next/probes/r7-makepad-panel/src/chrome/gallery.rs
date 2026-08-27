//! Chrome ギャラリー。部品を並べるだけ。Document を持たない。
//! splash 葉は登録済み `ChromeGallery{}` だけ。中身は `script_mod!` を `--hot` で付け替える。
//! `ScrollYView` は書かない。
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    mod.widgets.ChromeGalleryBase = #(ChromeGallery::register_widget(vm))
    mod.widgets.ChromeGallery = set_type_default() do mod.widgets.ChromeGalleryBase{
        width: Fill
        height: Fill
        flow: Down
        show_bg: true
        new_batch: true
        draw_bg.color: #x363636

        gallery_error := InkLabel{width: Fill height: Fit padding: 8 text: "" draw_text.color: #xe8c48a draw_text.text_style: theme.font_code{font_size: 10}}
        chrome_head := SolidView{width: Fill height: 26 flow: Right align: Align{y: 0.5} padding: Inset{left: 8 right: 8} show_bg: true new_batch: true draw_bg.color: #x2f2f2f
            title := InkLabel{text: "Chrome" width: Fill draw_text.color: #xcfcfcf draw_text.text_style: theme.font_bold{font_size: 11}}
        }
        chrome_body := View{
            width: Fill
            height: Fit
            flow: Down
            padding: 8
            spacing: 6
            new_batch: true

            face_cap := ChromeInkMicro{text: "FACE"}
            face_row := View{width: Fill height: Fit flow: Right spacing: 8
                ChromeFaceApp{height: 36 ChromeInk{text: "app"}}
                ChromeFace{height: 36 ChromeInk{text: "panel"}}
                ChromeFaceRaised{height: 36 ChromeInk{text: "raised"}}
            }

            ink_cap := ChromeInkMicro{text: "INK"}
            ink_row := View{width: Fill height: Fit flow: Right spacing: 8
                ChromeInkTitle{text: "title 12"}
                ChromeInk{text: "body 11"}
                ChromeInkCaption{text: "caption 9"}
                ChromeInkMicro{text: "micro 8"}
            }

            rule_cap := ChromeInkMicro{text: "RULE / ROW"}
            ChromeRule{}
            // 行高 16(Ableton 実測)に踏面 24 の ChromeButton は入らない。
            // 入れると潰れて文字が切れるので、ボタンは下の BUTTON 節で見せる
            row_sample := ChromeRow{
                ChromeInk{text: "Row label"}
            }

            button_cap := ChromeInkMicro{text: "BUTTON"}
            button_row := View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                ChromeButton{text: "Idle"}
                ChromeIcon{draw_icon +: {svg: crate_resource("self://resources/icons/lock.svg")}}
                ChromeGhost{draw_icon +: {svg: crate_resource("self://resources/icons/mute.svg")}}
            }

            toggle_cap := ChromeInkMicro{text: "TOGGLE"}
            toggle_row := View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                ChromeCheck{text: "Check"}
                ChromeToggle{text: "On"}
                ChromeLock{}
            }

            fold_cap := ChromeInkMicro{text: "FOLD"}
            fold_row := View{width: Fill height: Fit flow: Down spacing: 2
                ChromeTreeRow{title.text: "Layer"}
                ChromeFold{}
            }

            number_cap := ChromeInkMicro{text: "NUMBER"}
            ChromeScrub{text: "Opacity"}
            ChromeStepper{value.text: "24"}
            ChromeProgress{default: 0.4}

            color_cap := ChromeInkMicro{text: "COLOR"}
            color_row := View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                ChromeSwatch{}
                ChromeColorField{label.text: "Fill"}
                ChromeColorField{label.text: "Stroke"}
            }

            search_cap := ChromeInkMicro{text: "SEARCH"}
            ChromeSearch{}

            nav_cap := ChromeInkMicro{text: "NAV"}
            ChromeTabStrip{
                ChromeTabOn{tab.text: "Media"}
                ChromeTab{text: "Effects"}
                ChromeTab{text: "Create"}
            }
            ChromeChipStrip{
                ChromeChipOn{text: "All"}
                ChromeChip{text: "Video"}
                ChromeChip{text: "Audio"}
            }
            nav_rail := View{width: Fill height: Fit flow: Right
                ChromeRail{
                    ChromeRailItemOn{item.text: "Media"}
                    ChromeRailItem{text: "Effects"}
                    ChromeRailItem{text: "Create"}
                }
            }

            menu_cap := ChromeInkMicro{text: "MENU"}
            menu_row := View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                ChromeMenuBar{text: "File"}
                ChromeDrop{labels: ["Item" "Alt"]}
            }
            ChromeMenuFace{
                ChromeMenuLeaf{label.text: "New" hint.text: "Cmd+N"}
                ChromeMenuRule{}
                ChromeMenuItem{text: "Open…"}
            }

            transport_cap := ChromeInkMicro{text: "TRANSPORT"}
            transport_row := View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                ChromePlayhead{}
                ChromeProgressReadout{done.text: "42" total.text: "100" pct.text: "(42%)"}
            }
            ChromeTransport{}

            feedback_cap := ChromeInkMicro{text: "FEEDBACK"}
            feedback_row := View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                ChromeBadge{label.text: "KEY"}
                ChromeTooltip{face.label.text: "Hint"}
                ChromeStatus{label.text: "Ready"}
            }
            ChromeEmpty{headline.text: "Nothing selected" hint.text: "Select a layer."}
        }
    }
}

#[derive(Script, ScriptHook, WidgetRegister)]
pub struct ChromeGallery {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
}

impl WidgetNode for ChromeGallery {
    fn widget_uid(&self) -> WidgetUid {
        self.uid
    }

    fn children(&self, visit: &mut dyn FnMut(LiveId, WidgetRef)) {
        self.view.children(visit);
    }

    fn walk(&mut self, cx: &mut Cx) -> Walk {
        self.view.walk(cx)
    }

    fn area(&self) -> Area {
        self.view.area()
    }

    fn redraw(&mut self, cx: &mut Cx) {
        self.view.redraw(cx);
    }
}

impl Widget for ChromeGallery {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        if matches!(event, Event::LiveEdit) {
            self.refuse_empty(cx);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl ChromeGallery {
    fn refuse_empty(&mut self, cx: &mut Cx) {
        self.view.walk = Walk::fill();
        self.view.layout.flow = Flow::Down;
        let body_missing = self.view.child_by_path(ids!(chrome_body)).is_empty();
        let error = self.view.child_by_path(ids!(gallery_error)).as_label();
        if body_missing || self.view.children.is_empty() {
            if !error.is_empty() {
                error.set_text(cx, "Chrome を差し替えられない");
            }
        } else if !error.is_empty() {
            error.set_text(cx, "");
        }
        cx.redraw_all();
    }
}
