//! Tests for the `structuredClone` extension.

use crate::test::{TestAction, run_test_actions};

#[test]
fn clones_error_objects() {
    run_test_actions([TestAction::assert(
        r#"
        const original = new Error("boom");
        const cloned = structuredClone(original);

        cloned instanceof Error &&
            cloned !== original &&
            cloned.name === "Error" &&
            cloned.message === "boom";
    "#,
    )]);
}

#[test]
fn clones_native_error_subclasses() {
    run_test_actions([TestAction::assert(
        r#"
        const errors = [
            new TypeError("bad"),
            new RangeError("big"),
            new EvalError("ev"),
            new ReferenceError("rf"),
            new SyntaxError("sx"),
            new URIError("ur"),
        ];

        errors.every((original) => {
            const cloned = structuredClone(original);
            return (
                cloned instanceof original.constructor &&
                cloned.name === original.name &&
                cloned.message === original.message
            );
        });
    "#,
    )]);
}

#[test]
fn clones_error_object_cause() {
    run_test_actions([TestAction::assert(
        r#"
        const original = new Error("boom", { cause: { code: 7 } });
        const cloned = structuredClone(original);

        cloned instanceof Error &&
            cloned.cause !== original.cause &&
            cloned.cause.code === 7;
    "#,
    )]);
}

#[test]
fn clones_error_with_undefined_cause_property() {
    run_test_actions([TestAction::assert(
        r#"
        const original = new Error("boom", { cause: undefined });
        const cloned = structuredClone(original);

        Object.hasOwn(original, "cause") &&
            Object.hasOwn(cloned, "cause") &&
            cloned.cause === undefined;
    "#,
    )]);
}

#[test]
fn clones_error_with_circular_cause() {
    run_test_actions([TestAction::assert(
        r#"
        const original = new Error("boom");
        original.cause = original;

        const cloned = structuredClone(original);
        cloned !== original && cloned.cause === cloned;
    "#,
    )]);
}

#[test]
fn clones_error_stack() {
    run_test_actions([TestAction::assert(
        r#"
        const original = new Error("boom");
        const cloned = structuredClone(original);

        typeof original.stack === "string" &&
            typeof cloned.stack === "string" &&
            cloned.stack === original.stack;
    "#,
    )]);
}

#[test]
fn clones_aggregate_error_entries() {
    run_test_actions([TestAction::assert(
        r#"
        const original = new AggregateError([new Error("inner")], "agg");
        const cloned = structuredClone(original);

        cloned instanceof AggregateError &&
            cloned.message === "agg" &&
            cloned.errors.length === 1 &&
            cloned.errors[0] instanceof Error &&
            cloned.errors[0].message === "inner";
    "#,
    )]);
}

#[test]
fn clones_error_with_custom_name() {
    run_test_actions([TestAction::assert(
        r#"
        const original = new Error("boom");
        original.name = "CustomError";

        const cloned = structuredClone(original);
        cloned instanceof Error &&
            cloned.name === "CustomError" &&
            cloned.message === "boom";
    "#,
    )]);
}

#[test]
fn clones_error_nested_in_object() {
    run_test_actions([TestAction::assert(
        r#"
        const original = { error: new Error("boom") };
        const cloned = structuredClone(original);

        cloned.error instanceof Error &&
            cloned.error !== original.error &&
            cloned.error.message === "boom";
    "#,
    )]);
}

#[test]
fn functions_are_still_rejected() {
    run_test_actions([TestAction::assert(
        r#"
        try {
            structuredClone(() => {});
            false;
        } catch (e) {
            e.message.includes("unsupported type");
        }
    "#,
    )]);
}
