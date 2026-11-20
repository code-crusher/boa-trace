//! Exotic objects whose property internal methods are implemented by the host.
//!
//! Some host objects need to behave as if they had properties that are not stored in the
//! object itself. A DOM `NodeList`, for example, exposes an own property for every node it
//! contains, and an `HTMLCollection` also exposes properties named after the `id` and `name`
//! attributes of its elements. Those properties are computed from the host data every time
//! they are accessed, so they cannot be modelled with a plain [`JsObject`].
//!
//! A JavaScript `Proxy` can intercept the same operations, but it wraps a separate target
//! object: every operation has to be forwarded to the target, the target invariants have to be
//! upheld, and the object the host actually stores its data in is not the object the script
//! sees. [`JsNativeProxy`] instead makes the host data the object itself, and only lets the
//! host override the property internal methods.

use std::marker::PhantomData;

use boa_gc::{Finalize, Trace};

use crate::{
    Context, JsResult, JsValue,
    object::{
        JsObject, NativeObject,
        internal_methods::{
            InternalMethodPropertyContext, InternalObjectMethods, ORDINARY_INTERNAL_METHODS,
            ordinary_define_own_property, ordinary_delete, ordinary_get, ordinary_get_own_property,
            ordinary_has_property, ordinary_own_property_keys, ordinary_set,
        },
        shape::slot::SlotAttributes,
    },
    property::{CompletePropertyDescriptor, PropertyDescriptor, PropertyKey},
};

/// The property internal methods of a [`JsNativeProxy`].
///
/// Implementors can only override the internal methods that deal with properties. Every hook
/// defaults to the corresponding ordinary operation on the object's own properties, so an
/// implementor that overrides nothing behaves exactly like a plain object. The remaining
/// internal methods (prototype, extensibility, call and construct) cannot be overridden, which
/// keeps the object's structure and its prototype chain under the engine's control.
///
/// Note that the [invariants of the essential internal methods][invariants] are not checked,
/// since there is no second object to compare against; upholding them is up to the implementor.
///
/// # Examples
///
/// A collection whose elements are computed on every access:
///
/// ```
/// use boa_engine::{
///     Context, JsData, JsResult, JsValue, Source, js_string,
///     object::builtins::{JsNativeProxy, NativeProxyHooks},
///     property::{CompletePropertyDescriptor, PropertyKey},
/// };
/// use boa_gc::{Finalize, Trace};
///
/// #[derive(Trace, Finalize, JsData)]
/// struct NodeList {
///     #[unsafe_ignore_trace]
///     nodes: Vec<&'static str>,
/// }
///
/// impl NativeProxyHooks for NodeList {
///     fn get_own_property(
///         this: &JsNativeProxy<Self>,
///         key: &PropertyKey,
///         context: &mut Context,
///     ) -> JsResult<Option<CompletePropertyDescriptor>> {
///         if let PropertyKey::Index(index) = key {
///             let node = this.borrow().data().nodes.get(index.get() as usize).copied();
///             return Ok(node.map(|node| CompletePropertyDescriptor::Data {
///                 value: js_string!(node).into(),
///                 writable: false,
///                 enumerable: true,
///                 configurable: true,
///             }));
///         }
///
///         this.ordinary_get_own_property(key, context)
///     }
/// }
///
/// let context = &mut Context::default();
/// let list = JsNativeProxy::new(
///     context.intrinsics().constructors().object().prototype(),
///     NodeList { nodes: vec!["a", "b"] },
/// );
/// context
///     .global_object()
///     .set(js_string!("list"), list, false, context)?;
///
/// let result = context.eval(Source::from_bytes("list[0] + list[1] + list[2]"))?;
/// assert_eq!(result, JsValue::from(js_string!("abundefined")));
/// # Ok::<(), boa_engine::JsError>(())
/// ```
///
/// [invariants]: https://tc39.es/ecma262/#sec-invariants-of-the-essential-internal-methods
pub trait NativeProxyHooks: NativeObject + Sized {
    /// The `[[GetOwnProperty]]` internal method.
    fn get_own_property(
        this: &JsNativeProxy<Self>,
        key: &PropertyKey,
        context: &mut Context,
    ) -> JsResult<Option<CompletePropertyDescriptor>> {
        this.ordinary_get_own_property(key, context)
    }

    /// The `[[DefineOwnProperty]]` internal method.
    fn define_own_property(
        this: &JsNativeProxy<Self>,
        key: &PropertyKey,
        desc: PropertyDescriptor,
        context: &mut Context,
    ) -> JsResult<bool> {
        this.ordinary_define_own_property(key, desc, context)
    }

    /// The `[[HasProperty]]` internal method.
    fn has_property(
        this: &JsNativeProxy<Self>,
        key: &PropertyKey,
        context: &mut Context,
    ) -> JsResult<bool> {
        this.ordinary_has_property(key, context)
    }

    /// The `[[Get]]` internal method.
    fn get(
        this: &JsNativeProxy<Self>,
        key: &PropertyKey,
        receiver: JsValue,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        this.ordinary_get(key, receiver, context)
    }

    /// The `[[Set]]` internal method.
    fn set(
        this: &JsNativeProxy<Self>,
        key: PropertyKey,
        value: JsValue,
        receiver: JsValue,
        context: &mut Context,
    ) -> JsResult<bool> {
        this.ordinary_set(key, value, receiver, context)
    }

    /// The `[[Delete]]` internal method.
    fn delete(
        this: &JsNativeProxy<Self>,
        key: &PropertyKey,
        context: &mut Context,
    ) -> JsResult<bool> {
        this.ordinary_delete(key, context)
    }

    /// The `[[OwnPropertyKeys]]` internal method.
    fn own_property_keys(
        this: &JsNativeProxy<Self>,
        context: &mut Context,
    ) -> JsResult<Vec<PropertyKey>> {
        this.ordinary_own_property_keys(context)
    }
}

/// An object whose property internal methods are the [`NativeProxyHooks`] of `T`.
///
/// This is equivalent to the specification's `OrdinaryObjectCreate`, with the difference that
/// the object's own properties are handled by `T` instead of the ordinary internal methods.
///
/// The only way to construct this type is [`JsNativeProxy::new`].
#[derive(Trace, Finalize)]
pub struct JsNativeProxy<T: NativeProxyHooks> {
    inner: JsObject<T>,
}

impl<T: NativeProxyHooks> JsNativeProxy<T> {
    /// Creates a new exotic object with the provided prototype and host data.
    ///
    /// # Examples
    ///
    /// ```
    /// # use boa_engine::{Context, JsData, js_string};
    /// # use boa_engine::object::builtins::{JsNativeProxy, NativeProxyHooks};
    /// # use boa_gc::{Finalize, Trace};
    /// #[derive(Trace, Finalize, JsData)]
    /// struct Empty;
    ///
    /// impl NativeProxyHooks for Empty {}
    ///
    /// let context = &mut Context::default();
    /// let object = JsNativeProxy::new(
    ///     context.intrinsics().constructors().object().prototype(),
    ///     Empty,
    /// );
    ///
    /// // An implementor that overrides nothing is an ordinary object.
    /// object.set(js_string!("a"), 1, false, context)?;
    /// assert_eq!(object.get(js_string!("a"), context)?, 1.into());
    /// # Ok::<(), boa_engine::JsError>(())
    /// ```
    pub fn new<O: Into<Option<JsObject>>>(prototype: O, data: T) -> Self {
        let object = JsObject::new_unique_with_internal_methods(
            prototype,
            data,
            &NativeProxyMethods::<T>::VTABLE,
        );
        Self { inner: object }
    }

    /// Returns the object with its data typed as `T`.
    #[inline]
    #[must_use]
    pub fn as_object(&self) -> &JsObject<T> {
        &self.inner
    }

    /// Returns the object with its type erased.
    #[inline]
    #[must_use]
    pub fn upcast(&self) -> JsObject {
        self.inner.clone().upcast()
    }

    /// The ordinary `[[GetOwnProperty]]` of this object.
    ///
    /// Useful to fall back to the ordinary behavior from a [`NativeProxyHooks`] implementation.
    pub fn ordinary_get_own_property(
        &self,
        key: &PropertyKey,
        context: &mut Context,
    ) -> JsResult<Option<CompletePropertyDescriptor>> {
        ordinary_get_own_property(&self.upcast(), key, &mut context.into())
    }

    /// The ordinary `[[DefineOwnProperty]]` of this object.
    ///
    /// Useful to fall back to the ordinary behavior from a [`NativeProxyHooks`] implementation.
    pub fn ordinary_define_own_property(
        &self,
        key: &PropertyKey,
        desc: PropertyDescriptor,
        context: &mut Context,
    ) -> JsResult<bool> {
        ordinary_define_own_property(&self.upcast(), key, desc, &mut context.into())
    }

    /// The ordinary `[[HasProperty]]` of this object.
    ///
    /// Useful to fall back to the ordinary behavior from a [`NativeProxyHooks`] implementation.
    pub fn ordinary_has_property(
        &self,
        key: &PropertyKey,
        context: &mut Context,
    ) -> JsResult<bool> {
        ordinary_has_property(&self.upcast(), key, &mut context.into())
    }

    /// The ordinary `[[Get]]` of this object.
    ///
    /// Useful to fall back to the ordinary behavior from a [`NativeProxyHooks`] implementation.
    pub fn ordinary_get(
        &self,
        key: &PropertyKey,
        receiver: JsValue,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        ordinary_get(&self.upcast(), key, receiver, &mut context.into())
    }

    /// The ordinary `[[Set]]` of this object.
    ///
    /// Useful to fall back to the ordinary behavior from a [`NativeProxyHooks`] implementation.
    pub fn ordinary_set(
        &self,
        key: PropertyKey,
        value: JsValue,
        receiver: JsValue,
        context: &mut Context,
    ) -> JsResult<bool> {
        ordinary_set(&self.upcast(), key, value, receiver, &mut context.into())
    }

    /// The ordinary `[[Delete]]` of this object.
    ///
    /// Useful to fall back to the ordinary behavior from a [`NativeProxyHooks`] implementation.
    pub fn ordinary_delete(&self, key: &PropertyKey, context: &mut Context) -> JsResult<bool> {
        ordinary_delete(&self.upcast(), key, &mut context.into())
    }

    /// The ordinary `[[OwnPropertyKeys]]` of this object.
    ///
    /// Useful to fall back to the ordinary behavior from a [`NativeProxyHooks`] implementation.
    pub fn ordinary_own_property_keys(&self, context: &mut Context) -> JsResult<Vec<PropertyKey>> {
        ordinary_own_property_keys(&self.upcast(), context)
    }
}

impl<T: NativeProxyHooks> Clone for JsNativeProxy<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T: NativeProxyHooks> std::fmt::Debug for JsNativeProxy<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.inner, f)
    }
}

impl<T: NativeProxyHooks> std::ops::Deref for JsNativeProxy<T> {
    type Target = JsObject<T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: NativeProxyHooks> From<JsNativeProxy<T>> for JsObject<T> {
    #[inline]
    fn from(object: JsNativeProxy<T>) -> Self {
        object.inner.clone()
    }
}

impl<T: NativeProxyHooks> From<JsNativeProxy<T>> for JsObject {
    #[inline]
    fn from(object: JsNativeProxy<T>) -> Self {
        object.inner.clone().upcast()
    }
}

impl<T: NativeProxyHooks> From<JsNativeProxy<T>> for JsValue {
    #[inline]
    fn from(object: JsNativeProxy<T>) -> Self {
        object.inner.clone().upcast().into()
    }
}

/// The [`InternalObjectMethods`] of a [`JsNativeProxy`].
struct NativeProxyMethods<T>(PhantomData<T>);

impl<T: NativeProxyHooks> NativeProxyMethods<T> {
    /// The internal methods that dispatch to the hooks of `T`.
    ///
    /// Only the property internal methods are overridden; everything else is ordinary.
    pub(crate) const VTABLE: InternalObjectMethods = InternalObjectMethods {
        __get_own_property__: native_proxy_get_own_property::<T>,
        __define_own_property__: native_proxy_define_own_property::<T>,
        __has_property__: native_proxy_has_property::<T>,
        __try_get__: native_proxy_try_get::<T>,
        __get__: native_proxy_get::<T>,
        __set__: native_proxy_set::<T>,
        __delete__: native_proxy_delete::<T>,
        __own_property_keys__: native_proxy_own_property_keys::<T>,
        ..ORDINARY_INTERNAL_METHODS
    };
}

/// Casts the erased object back to the type its hooks expect.
///
/// The engine only installs these internal methods through [`NativeProxyMethods::VTABLE`],
/// which is only reachable from [`JsNativeProxy::new`], so the cast always succeeds.
#[track_caller]
fn typed<T: NativeProxyHooks>(object: &JsObject) -> JsNativeProxy<T> {
    let inner = object
        .clone()
        .downcast::<T>()
        .unwrap_or_else(|_| unreachable!("native proxy internal method called on another type"));
    JsNativeProxy { inner }
}

/// Runs a hook and marks the property access as non-cacheable.
///
/// A hook can return a different value on every call, so the result of a property access on
/// this object must never be cached in the object's shape. The mark is applied after the hook
/// runs because hooks that fall back to an ordinary operation reset the slot's attributes.
fn run_hook<R>(
    context: &mut InternalMethodPropertyContext<'_>,
    hook: impl FnOnce(&mut InternalMethodPropertyContext<'_>) -> JsResult<R>,
) -> JsResult<R> {
    let result = hook(context);
    context.slot().attributes |= SlotAttributes::NOT_CACHEABLE;
    result
}

fn native_proxy_get_own_property<T: NativeProxyHooks>(
    object: &JsObject,
    key: &PropertyKey,
    context: &mut InternalMethodPropertyContext<'_>,
) -> JsResult<Option<CompletePropertyDescriptor>> {
    let this = typed::<T>(object);
    run_hook(context, |context| {
        T::get_own_property(&this, key, context)
    })
}

fn native_proxy_define_own_property<T: NativeProxyHooks>(
    object: &JsObject,
    key: &PropertyKey,
    desc: PropertyDescriptor,
    context: &mut InternalMethodPropertyContext<'_>,
) -> JsResult<bool> {
    let this = typed::<T>(object);
    run_hook(context, |context| {
        T::define_own_property(&this, key, desc, context)
    })
}

fn native_proxy_has_property<T: NativeProxyHooks>(
    object: &JsObject,
    key: &PropertyKey,
    context: &mut InternalMethodPropertyContext<'_>,
) -> JsResult<bool> {
    let this = typed::<T>(object);
    run_hook(context, |context| T::has_property(&this, key, context))
}

/// The combined `[[HasProperty]]` and `[[Get]]` used by the engine's property lookups.
fn native_proxy_try_get<T: NativeProxyHooks>(
    object: &JsObject,
    key: &PropertyKey,
    receiver: JsValue,
    context: &mut InternalMethodPropertyContext<'_>,
) -> JsResult<Option<JsValue>> {
    let this = typed::<T>(object);
    run_hook(context, |context| {
        if T::has_property(&this, key, context)? {
            Ok(Some(T::get(&this, key, receiver, context)?))
        } else {
            Ok(None)
        }
    })
}

fn native_proxy_get<T: NativeProxyHooks>(
    object: &JsObject,
    key: &PropertyKey,
    receiver: JsValue,
    context: &mut InternalMethodPropertyContext<'_>,
) -> JsResult<JsValue> {
    let this = typed::<T>(object);
    run_hook(context, |context| {
        T::get(&this, key, receiver, context)
    })
}

fn native_proxy_set<T: NativeProxyHooks>(
    object: &JsObject,
    key: PropertyKey,
    value: JsValue,
    receiver: JsValue,
    context: &mut InternalMethodPropertyContext<'_>,
) -> JsResult<bool> {
    let this = typed::<T>(object);
    run_hook(context, |context| {
        T::set(&this, key, value, receiver, context)
    })
}

fn native_proxy_delete<T: NativeProxyHooks>(
    object: &JsObject,
    key: &PropertyKey,
    context: &mut InternalMethodPropertyContext<'_>,
) -> JsResult<bool> {
    let this = typed::<T>(object);
    run_hook(context, |context| T::delete(&this, key, context))
}

fn native_proxy_own_property_keys<T: NativeProxyHooks>(
    object: &JsObject,
    context: &mut Context,
) -> JsResult<Vec<PropertyKey>> {
    T::own_property_keys(&typed(object), context)
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use boa_gc::{Finalize, Trace};
    use indoc::indoc;
    use rustc_hash::FxHashMap;

    use super::{JsNativeProxy, NativeProxyHooks};
    use crate::{
        Context, JsData, JsNativeError, JsNativeErrorKind, JsResult, JsString, JsValue, TestAction,
        js_string,
        native_function::NativeFunction,
        object::{JsObject, ObjectInitializer},
        property::{CompletePropertyDescriptor, PropertyDescriptor, PropertyKey},
        run_test_actions,
    };

    /// A collection with named properties, like an `HTMLCollection`.
    #[derive(Default, Trace, Finalize, JsData)]
    struct NamedCollection {
        #[unsafe_ignore_trace]
        entries: FxHashMap<JsString, JsString>,
        #[unsafe_ignore_trace]
        order: Vec<JsString>,
    }

    impl NamedCollection {
        fn insert(&mut self, key: JsString, value: JsString) {
            if self.entries.insert(key.clone(), value).is_none() {
                self.order.push(key);
            }
        }

        fn remove(&mut self, key: &JsString) -> bool {
            if self.entries.remove(key).is_some() {
                self.order.retain(|k| k != key);
                true
            } else {
                false
            }
        }
    }

    impl NativeProxyHooks for NamedCollection {
        fn get_own_property(
            this: &JsNativeProxy<Self>,
            key: &PropertyKey,
            context: &mut Context,
        ) -> JsResult<Option<CompletePropertyDescriptor>> {
            if let PropertyKey::String(name) = key
                && let Some(value) = this.borrow().data().entries.get(name)
            {
                return Ok(Some(CompletePropertyDescriptor::Data {
                    value: value.clone().into(),
                    writable: true,
                    enumerable: true,
                    configurable: true,
                }));
            }
            this.ordinary_get_own_property(key, context)
        }

        fn define_own_property(
            this: &JsNativeProxy<Self>,
            key: &PropertyKey,
            desc: PropertyDescriptor,
            context: &mut Context,
        ) -> JsResult<bool> {
            if let PropertyKey::String(name) = key
                && let Some(value) = desc.value()
            {
                let value = value.to_string(context)?;
                this.borrow_mut().data_mut().insert(name.clone(), value);
                return Ok(true);
            }
            this.ordinary_define_own_property(key, desc, context)
        }

        fn delete(
            this: &JsNativeProxy<Self>,
            key: &PropertyKey,
            context: &mut Context,
        ) -> JsResult<bool> {
            if let PropertyKey::String(name) = key
                && this.borrow_mut().data_mut().remove(name)
            {
                return Ok(true);
            }
            this.ordinary_delete(key, context)
        }

        fn own_property_keys(
            this: &JsNativeProxy<Self>,
            context: &mut Context,
        ) -> JsResult<Vec<PropertyKey>> {
            let mut keys = this.ordinary_own_property_keys(context)?;
            keys.extend(
                this.borrow()
                    .data()
                    .order
                    .iter()
                    .cloned()
                    .map(PropertyKey::from),
            );
            Ok(keys)
        }
    }

    /// An object whose `next` property returns a new value on every read.
    #[derive(Default, Trace, Finalize, JsData)]
    struct Counter {
        #[unsafe_ignore_trace]
        count: Cell<u32>,
    }

    impl NativeProxyHooks for Counter {
        fn has_property(
            this: &JsNativeProxy<Self>,
            key: &PropertyKey,
            context: &mut Context,
        ) -> JsResult<bool> {
            if key == &PropertyKey::from(js_string!("next")) {
                return Ok(true);
            }
            this.ordinary_has_property(key, context)
        }

        fn get(
            this: &JsNativeProxy<Self>,
            key: &PropertyKey,
            receiver: JsValue,
            context: &mut Context,
        ) -> JsResult<JsValue> {
            if key == &PropertyKey::from(js_string!("next")) {
                let data = this.borrow();
                let count = &data.data().count;
                count.set(count.get() + 1);
                return Ok(count.get().into());
            }
            this.ordinary_get(key, receiver, context)
        }
    }

    /// An object that doesn't override any hook.
    #[derive(Trace, Finalize, JsData)]
    struct Plain;

    impl NativeProxyHooks for Plain {}

    fn size(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        let size = this
            .as_object()
            .and_then(|object| object.downcast_ref::<NamedCollection>().map(|c| c.entries.len()))
            .ok_or_else(|| JsNativeError::typ().with_message("Illegal invocation"))?;
        Ok(size.into())
    }

    fn register(context: &mut Context) {
        let object_prototype = context.intrinsics().constructors().object().prototype();

        let prototype = ObjectInitializer::new(context)
            .function(NativeFunction::from_fn_ptr(size), js_string!("size"), 0)
            .build();
        prototype.set_prototype(Some(object_prototype.clone()));

        let collection = JsNativeProxy::new(prototype.clone(), NamedCollection::default());
        let counter = JsNativeProxy::new(object_prototype.clone(), Counter::default());
        let plain = JsNativeProxy::new(object_prototype, Plain);

        let global = context.global_object();
        for (name, value) in [
            (js_string!("collection"), JsValue::from(collection)),
            (js_string!("collectionPrototype"), prototype.into()),
            (js_string!("counter"), counter.into()),
            (js_string!("plain"), plain.into()),
        ] {
            global
                .define_property_or_throw(
                    name,
                    PropertyDescriptor::builder()
                        .value(value)
                        .writable(true)
                        .enumerable(false)
                        .configurable(true),
                    context,
                )
                .expect("should define the test globals");
        }
    }

    fn collection_object(context: &mut Context) -> JsObject {
        context
            .global_object()
            .get(js_string!("collection"), context)
            .expect("collection should be defined")
            .as_object()
            .expect("collection should be an object")
            .clone()
    }

    #[test]
    fn named_properties() {
        run_test_actions([
            TestAction::run_harness(),
            TestAction::inspect_context(register),
            TestAction::run(indoc! {r#"
                collection.a = 1;
                collection["b"] = { toString() { return "two"; } };
                Object.defineProperty(collection, "c", { value: 3 });
            "#}),
            TestAction::assert_eq("collection.a", js_string!("1")),
            TestAction::assert_eq("collection.b", js_string!("two")),
            TestAction::assert("'c' in collection"),
            TestAction::assert(r#"arrayEquals(Object.keys(collection), ["a", "b", "c"])"#),
            TestAction::assert(
                r#"JSON.stringify(collection) === '{"a":"1","b":"two","c":"3"}'"#,
            ),
            TestAction::assert("delete collection.b"),
            TestAction::assert("!('b' in collection) && collection.b === undefined"),
            TestAction::assert(
                r#"arrayEquals(Object.getOwnPropertyNames(collection), ["a", "c"])"#,
            ),
            TestAction::inspect_context(|context| {
                let collection = collection_object(context);
                let data = collection
                    .downcast_ref::<NamedCollection>()
                    .expect("collection should be a NamedCollection");
                assert_eq!(data.order, [js_string!("a"), js_string!("c")]);
            }),
        ]);
    }

    #[test]
    fn prototype_methods_see_the_data() {
        run_test_actions([
            TestAction::inspect_context(register),
            TestAction::run("collection.x = 'x'; collection.y = 'y'"),
            TestAction::assert_eq("collection.size()", 2),
            TestAction::assert_native_error(
                "collectionPrototype.size.call({})",
                JsNativeErrorKind::Type,
                "Illegal invocation",
            ),
        ]);
    }

    #[test]
    fn hooks_are_not_cached() {
        run_test_actions([
            TestAction::run_harness(),
            TestAction::inspect_context(register),
            TestAction::run(indoc! {r#"
                function read(o) { return o.next; }
                var direct = [];
                var inherited = [];
                var child = Object.create(counter);
                for (let i = 0; i < 3; i++) {
                    direct.push(read(counter));
                }
                for (let i = 0; i < 3; i++) {
                    inherited.push(read(child));
                }
            "#}),
            TestAction::assert("arrayEquals(direct, [1, 2, 3])"),
            TestAction::assert("arrayEquals(inherited, [4, 5, 6])"),
            TestAction::run(indoc! {r#"
                function readA(o) { return o.a; }
                var values = [];
                for (let i = 0; i < 3; i++) {
                    collection.a = "v" + i;
                    values.push(readA(collection));
                }
            "#}),
            TestAction::assert(r#"arrayEquals(values, ["v0", "v1", "v2"])"#),
        ]);
    }

    #[test]
    fn with_statement_uses_has_and_get() {
        run_test_actions([
            TestAction::inspect_context(register),
            TestAction::assert_eq("with (counter) { next + next }", 3),
        ]);
    }

    #[test]
    fn default_hooks_are_ordinary() {
        run_test_actions([
            TestAction::run_harness(),
            TestAction::inspect_context(register),
            TestAction::run(indoc! {r#"
                plain.a = 1;
                Object.defineProperty(plain, "fixed", { value: 2 });
                plain[0] = "zero";
            "#}),
            TestAction::assert_eq("plain.a + plain.fixed", 3),
            TestAction::assert(r#"arrayEquals(Object.keys(plain), ["0", "a"])"#),
            TestAction::assert("!delete plain.fixed && delete plain.a && !('a' in plain)"),
            TestAction::assert_native_error(
                "'use strict'; plain.fixed = 3",
                JsNativeErrorKind::Type,
                "cannot set non-writable property: fixed",
            ),
        ]);
    }

    #[test]
    fn non_property_internal_methods_are_ordinary() {
        run_test_actions([
            TestAction::run_harness(),
            TestAction::inspect_context(register),
            // Prototypes and extensibility are not overridable by the hooks.
            TestAction::assert("Object.getPrototypeOf(counter) === Object.prototype"),
            TestAction::assert("Object.isExtensible(plain) && Object.isExtensible(counter)"),
            TestAction::assert("Object.preventExtensions(plain) && !Object.isExtensible(plain)"),
            // The object is neither callable nor constructible.
            TestAction::assert_native_error(
                "counter()",
                JsNativeErrorKind::Type,
                "not a callable function",
            ),
            TestAction::assert_native_error(
                "new counter()",
                JsNativeErrorKind::Type,
                "object is not a constructor",
            ),
        ]);
    }
}
