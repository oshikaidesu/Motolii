import { expect, test } from "@playwright/test";

const reactUrl = "http://127.0.0.1:5173/";
const route = `${reactUrl}#diagnostics/feedback-states`;

test.beforeEach(async ({ page }) => {
  await page.goto(route);
  await expect(
    page.getByRole("main", { name: "Common feedback state matrix" }),
  ).toBeVisible();
});

test("renders the exact nine-state matrix outside product catalogs", async ({
  page,
}) => {
  const cases = page.locator("[data-feedback-case]");
  await expect(cases).toHaveCount(9);
  expect(
    await cases.evaluateAll((nodes) =>
      nodes.map((node) => node.getAttribute("data-feedback-case")),
    ),
  ).toEqual([
      "inline-neutral",
      "target-valid",
      "target-invalid",
      "disabled-action",
      "warning",
      "error-unrecoverable",
      "loading",
      "semantic-badge",
      "cursor-context",
    ]);

  await page.goto(`${reactUrl}#catalog`);
  await expect(
    page.getByRole("main", { name: "Common feedback state matrix" }),
  ).toHaveCount(0);
  await page.goto(`${reactUrl}#archive/catalog`);
  await expect(
    page.getByRole("main", { name: "Common feedback state matrix" }),
  ).toHaveCount(0);
});

test("keeps typed reason and recovery visible and focus reachable", async ({
  page,
}) => {
  for (const id of [
    "target-invalid",
    "disabled-action",
    "warning",
    "error-unrecoverable",
    "cursor-context",
  ]) {
    const fixture = page.locator(`[data-feedback-case="${id}"]`);
    const feedback = fixture.locator(".motolii-feedback");
    await expect(feedback).toHaveAttribute("data-feedback-reason", /.+/);
    await expect(feedback).toHaveAttribute("data-feedback-recovery", /.+/);
    await expect(feedback.locator(".motolii-feedback__reason")).toBeVisible();
    await expect(feedback.locator(".motolii-feedback__recovery")).toBeVisible();
    await feedback.focus();
    await expect(feedback).toBeFocused();
    await expect(feedback).toHaveAttribute("aria-describedby", /.+/);
  }
});

test("uses structure and accessibility state in addition to color and text", async ({
  page,
}) => {
  for (const fixture of await page.locator("[data-feedback-case]").all()) {
    const feedback = fixture.locator(".motolii-feedback");
    await expect(feedback.locator(".motolii-feedback__marker")).toBeVisible();
    await expect(feedback.locator(".motolii-feedback__label")).not.toBeEmpty();
    await expect(feedback).toHaveAttribute("role", /^(group|status|alert)$/);
  }

  await expect(
    page.locator('[data-feedback-case="loading"] .motolii-feedback'),
  ).toHaveAttribute("aria-busy", "true");
  await expect(
    page.locator('[data-feedback-case="error-unrecoverable"] .motolii-feedback'),
  ).toHaveAttribute("role", "alert");
  await expect(
    page.locator('[data-feedback-case="target-valid"] .motolii-feedback'),
  ).toHaveCSS("border-top-style", "dashed");
  await expect(
    page.locator('[data-feedback-case="disabled-action"] .motolii-feedback'),
  ).toHaveCSS("border-top-style", "dashed");
});

test("fails closed when contextual feedback omits reason or recovery", async ({
  page,
}) => {
  const outcomes = await page.evaluate(async () => {
    const { validateFeedbackModel } = await import(
      "/src/feedback/Feedback.jsx"
    );
    const base = {
      placement: "inline",
      tone: "warning",
      label: "Cannot continue",
    };
    return [
      base,
      {
        ...base,
        reason: { code: "missing.recovery", text: "Recovery is missing" },
      },
      {
        ...base,
        recovery: {
          kind: "retry-with-changed-input",
          text: "Reason is missing",
        },
      },
    ].map((model) => {
      try {
        validateFeedbackModel(model);
        return "accepted";
      } catch (error) {
        return error instanceof TypeError ? "typed-reject" : "wrong-error";
      }
    });
  });

  expect(outcomes).toEqual([
    "typed-reject",
    "typed-reject",
    "typed-reject",
  ]);
});
