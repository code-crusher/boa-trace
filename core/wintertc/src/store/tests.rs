//! Tests for the [`JsValueStore`] conversions, focused on the two-phase
//! construction used to support recursive values.

use crate::store::{JsValueStore, ValueStoreInner};
use crate::test::{TestAction, run_test_actions_with};
use boa_engine::object::builtins::{JsMap, JsSet};
use boa_engine::value::TryIntoJs;
use boa_engine::{Context, JsString, JsValue, Source};
use indoc::indoc;

/// Evaluates `source` in `context`, then round-trips the resulting value
/// through a [`JsValueStore`] and returns the clone.
fn round_trip(source: &str, context: &mut Context) -> JsValue {
    let original = context
        .eval(Source::from_bytes(source))
        .expect("the test source should evaluate without errors");

    let store = JsValueStore::try_from_js(&original, context, Vec::new())
        .expect("the value should convert into a store");

    store
        .try_into_js(context)
        .expect("the store should convert back into a value")
}

/// Gets the property `name` of `object` from `context`.
fn get_prop(object: &JsValue, name: &str, context: &mut Context) -> JsValue {
    object
        .as_object()
        .expect("the value should be an object")
        .get(JsString::from(name), context)
        .expect("the property should be readable")
}

#[test]
fn plain_object_round_trip() {
    let context = &mut Context::default();

    let cloned = round_trip(
        r#"({ a: 1, b: "two", c: true, d: null, e: undefined, f: 4.5 })"#,
        context,
    );

    assert!(get_prop(&cloned, "a", context).strict_equals(&JsValue::from(1)));
    assert!(get_prop(&cloned, "b", context).strict_equals(&JsValue::from(JsString::from("two"))));
    assert!(get_prop(&cloned, "c", context).strict_equals(&JsValue::from(true)));
    assert!(get_prop(&cloned, "d", context).strict_equals(&JsValue::null()));
    assert!(get_prop(&cloned, "e", context).strict_equals(&JsValue::undefined()));
    assert!(get_prop(&cloned, "f", context).strict_equals(&JsValue::from(4.5)));
}

#[test]
fn sparse_array_round_trip() {
    let context = &mut Context::default();

    // The hole at index 1 must survive the round-trip.
    let cloned = round_trip("[10, , 30]", context);

    assert_eq!(
        get_prop(&cloned, "length", context).as_number(),
        Some(3.0),
        "the array length should be preserved"
    );
    assert!(get_prop(&cloned, "0", context).strict_equals(&JsValue::from(10)));
    assert!(
        get_prop(&cloned, "1", context).is_undefined(),
        "the hole at index 1 should stay a hole"
    );
    assert!(get_prop(&cloned, "2", context).strict_equals(&JsValue::from(30)));
}

#[test]
fn array_cycle_round_trip() {
    let context = &mut Context::default();

    let cloned = round_trip("const a = [1]; a.push(a); a", context);

    let inner = cloned
        .as_object()
        .expect("the value should be an object")
        .get(1usize, context)
        .expect("the element should be readable");

    assert!(
        inner.strict_equals(&cloned),
        "the cycle should point back to the clone itself"
    );
}

#[test]
fn object_cycle_round_trip() {
    let context = &mut Context::default();

    let cloned = round_trip("const o = { x: 1 }; o.self = o; o", context);

    assert!(get_prop(&cloned, "x", context).strict_equals(&JsValue::from(1)));

    let self_ref = get_prop(&cloned, "self", context);
    assert!(
        self_ref.strict_equals(&cloned),
        "the cycle should point back to the clone itself"
    );
}

#[test]
fn mutual_cycles_round_trip() {
    let context = &mut Context::default();

    // `a[1][0] === a`: two arrays that refer to each other.
    let cloned = round_trip("const a = [1]; const b = [a]; a.push(b); a", context);

    let b = cloned
        .as_object()
        .expect("the value should be an object")
        .get(1usize, context)
        .expect("the element should be readable");

    let a = b
        .as_object()
        .expect("the value should be an object")
        .get(0usize, context)
        .expect("the element should be readable");

    assert!(
        a.strict_equals(&cloned),
        "the mutual cycle should point back to the clone itself"
    );
}

#[test]
fn map_cycle_round_trip() {
    let context = &mut Context::default();

    // The map refers to itself in the value position.
    let cloned = round_trip(r#"const m = new Map(); m.set("self", m); m"#, context);

    let map = JsMap::from_object(cloned.as_object().expect("should be a map").clone())
        .expect("should be a map");

    let self_ref = map
        .get(JsString::from("self"), context)
        .expect("the entry should be readable");
    assert!(
        self_ref.strict_equals(&cloned),
        "the cycle should point back to the clone itself"
    );

    // ... and in the key position.
    let cloned = round_trip(r#"const m2 = new Map(); m2.set(m2, "value"); m2"#, context);

    let map = JsMap::from_object(cloned.as_object().expect("should be a map").clone())
        .expect("should be a map");

    let value = map
        .get(cloned.clone(), context)
        .expect("the entry should be readable");
    assert!(
        value.strict_equals(&JsValue::from(JsString::from("value"))),
        "the cycle key should look up the clone itself"
    );
}

#[test]
fn set_cycle_round_trip() {
    let context = &mut Context::default();

    // The set contains itself.
    let cloned = round_trip("const s = new Set(); s.add(s); s", context);

    let set = JsSet::from_object(cloned.as_object().expect("should be a set").clone())
        .expect("should be a set");

    assert_eq!(set.size(), 1, "the set should have a single element");
    assert!(
        set.has(cloned.clone()),
        "the set should contain the clone itself"
    );
}

#[test]
fn shared_references_round_trip() {
    let context = &mut Context::default();

    // `shared` is stored once and referenced twice; the clone must keep the
    // two references pointing at the same (new) object.
    let cloned = round_trip(
        r#"const shared = { x: 1 }; ({ a: shared, b: shared })"#,
        context,
    );

    let a = get_prop(&cloned, "a", context);
    let b = get_prop(&cloned, "b", context);

    assert!(
        a.strict_equals(&b),
        "both references should point to the same clone"
    );
    assert!(get_prop(&a, "x", context).strict_equals(&JsValue::from(1)));
}

#[test]
fn structured_clone_recursive() {
    let context = &mut Context::default();
    crate::clone::register(None, context).expect("failed to register structuredClone");

    run_test_actions_with(
        [TestAction::run(indoc! {r#"
            const o = { name: "outer" };
            o.self = o;
            const c = structuredClone(o);
            if (c === o) {
                throw new Error("the clone should be a new object");
            }
            if (c.self !== c) {
                throw new Error("the clone cycle should point to the clone");
            }
            if (c.name !== "outer") {
                throw new Error("scalar fields should survive");
            }

            const arr = [1];
            arr.push(arr);
            const arrClone = structuredClone(arr);
            if (arrClone[1] !== arrClone) {
                throw new Error("array cycles should point to the clone");
            }

            const shared = { x: 1 };
            const obj = { a: shared, b: shared };
            const objClone = structuredClone(obj);
            if (objClone.a !== objClone.b) {
                throw new Error("shared references should be cloned once");
            }

            const m = new Map();
            m.set("self", m);
            const mClone = structuredClone(m);
            if (mClone.get("self") !== mClone) {
                throw new Error("map cycles should point to the clone");
            }

            const s = new Set();
            s.add(s);
            const sClone = structuredClone(s);
            if (!sClone.has(sClone)) {
                throw new Error("set cycles should contain the clone");
            }

            if (structuredClone(10n) !== 10n) {
                throw new Error("bigints should survive");
            }
        "#})],
        context,
    );
}

#[test]
fn structured_clone_transfer() {
    let context = &mut Context::default();
    crate::clone::register(None, context).expect("failed to register structuredClone");

    run_test_actions_with(
        [TestAction::run(indoc! {r#"
            const buffer = new ArrayBuffer(4);
            const view = new Uint8Array(buffer);
            view[0] = 42;
            view[3] = 7;

            const clone = structuredClone(view, { transfer: [buffer] });
            if (clone[0] !== 42 || clone[3] !== 7) {
                throw new Error("transferred data should survive");
            }
            if (buffer.byteLength !== 0) {
                throw new Error("the original buffer should be detached");
            }
        "#})],
        context,
    );
}

#[test]
fn js_value_store_is_send_and_sync() {
    // `postMessage` moves `JsValueStore`s between threads, so they must stay
    // `Send + Sync`.
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<JsValueStore>();
}

#[test]
#[should_panic(expected = "ValueStoreInner must only be filled once")]
fn fill_twice_panics() {
    let store = JsValueStore::empty();
    store.fill(ValueStoreInner::Null);
    store.fill(ValueStoreInner::Null);
}
