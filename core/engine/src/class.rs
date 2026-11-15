//! Traits and structs for implementing native classes.
//!
//! Native classes are implemented through the [`Class`][class-trait] trait.
//!
//! # Examples
//!
//! ```
//! # use boa_engine::{
//! #    NativeFunction,
//! #    property::Attribute,
//! #    class::{Class, ClassBuilder},
//! #    Context, JsResult, JsValue,
//! #    JsArgs, Source, JsObject, js_str, js_string,
//! #    JsNativeError, JsData,
//! # };
//! # use boa_gc::{Finalize, Trace};
//! #
//! // Can also be a struct containing `Trace` types.
//! #[derive(Debug, Trace, Finalize, JsData)]
//! enum Animal {
//!     Cat,
//!     Dog,
//!     Other,
//! }
//!
//! impl Class for Animal {
//!     // we set the binding name of this function to be `"Animal"`.
//!     const NAME: &'static str = "Animal";
//!
//!     // We set the length to `2` since we accept 2 arguments in the constructor.
//!     const LENGTH: usize = 2;
//!
//!     // This is what is called when we do `new Animal()` to construct the inner data of the class.
//!     // `_new_target` is the target of the `new` invocation, in this case the `Animal` constructor
//!     // object.
//!     fn data_constructor(_new_target: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<Self> {
//!         // This is equivalent to `String(arg)`.
//!         let kind = args.get_or_undefined(0).to_string(context)?;
//!
//!         let animal = match kind.to_std_string_escaped().as_str() {
//!             "cat" => Self::Cat,
//!             "dog" => Self::Dog,
//!             _ => Self::Other,
//!         };
//!
//!         Ok(animal)
//!     }
//!
//!     // This is also called on instance construction, but it receives the object wrapping the
//!     // native data as its `instance` argument.
//!     fn object_constructor(
//!         instance: &JsObject,
//!         args: &[JsValue],
//!         context: &mut Context,
//!     ) -> JsResult<()> {
//!         let age = args.get_or_undefined(1).to_number(context)?;
//!
//!         // Roughly equivalent to `this.age = Number(age)`.
//!         instance.clone().set(js_string!("age"), age, true, context)?;
//!
//!         Ok(())
//!     }
//!
//!     /// This is where the class object is initialized.
//!     fn init(class: &mut ClassBuilder) -> JsResult<()> {
//!         class.method(
//!             js_string!("speak"),
//!             0,
//!             NativeFunction::from_fn_ptr(|this, _args, _ctx| {
//!                 if let Some(object) = this.as_object()
//!                     && let Ok(animal) = class::with_class_data::<Animal, _>(&object, |animal| {
//!                         Ok(match *animal {
//!                             Self::Cat => js_string!("meow"),
//!                             Self::Dog => js_string!("woof"),
//!                             Self::Other => js_string!(r"¯\_(ツ)_/¯"),
//!                         }
//!                         .into())
//!                     })
//!                 {
//!                     return animal;
//!                 }
//!                 Err(JsNativeError::typ().with_message("invalid this for class method").into())
//!             }),
//!         );
//!         Ok(())
//!     }
//! }
//!
//! fn main() {
//!     let mut context = Context::default();
//!
//!     context.register_global_class::<Animal>().unwrap();
//!
//!     let result = context.eval(Source::from_bytes(r#"
//!         let pet = new Animal("dog", 3);
//!
//!         `My pet is ${pet.age} years old. Right, buddy? - ${pet.speak()}!`
//!     "#)).unwrap();
//!
//!     assert_eq!(
//!         result.as_string().unwrap(),
//!         js_str!("My pet is 3 years old. Right, buddy? - woof!")
//!     );
//! }
//! ```
//!
//! [class-trait]: ./trait.Class.html

use crate::{
    Context, JsData, JsResult, JsValue,
    context::intrinsics::StandardConstructor,
    error::JsNativeError,
    native_function::NativeFunction,
    object::{ConstructorBuilder, FunctionBinding, JsFunction, JsObject, NativeObject, PROTOTYPE},
    property::{Attribute, CompletePropertyDescriptor, PropertyKey},
};
use boa_gc::{Finalize, GcRefCell, Trace};
use std::any::{Any, TypeId};

/// Sentinel type for native classes with no parent class.
///
/// This is the value of [`Class::Parent`] for root native classes, and it is never registered or
/// instantiated in practice.
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct NoParent;

impl Class for NoParent {
    const NAME: &'static str = "";
    // `DEPTH == 0` and `INDEX == 0` are the base cases of the native inheritance chain.
    const DEPTH: usize = 0;
    const INDEX: usize = 0;

    type Parent = NoParent;

    fn init(_class: &mut ClassBuilder<'_>) -> JsResult<()> {
        Ok(())
    }

    fn data_constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<Self> {
        Ok(NoParent)
    }
}

/// Native class.
///
/// See the [module-level documentation][self] for more details.
pub trait Class: NativeObject + Sized {
    /// The binding name of this class.
    const NAME: &'static str;
    /// The amount of arguments this class' constructor takes. Default is `0`.
    const LENGTH: usize = 0;
    /// The property attributes of this class' constructor in the global object.
    /// Default is `writable`, `enumerable`, `configurable`.
    const ATTRIBUTES: Attribute = Attribute::all();

    /// The parent class of this class in the native inheritance chain.
    ///
    /// Root native classes use [`NoParent`].
    ///
    /// When a class declares a parent, its prototype inherits from the parent's prototype and its
    /// constructor inherits from the parent's constructor. Instances additionally carry the native
    /// data of every class in the chain, so methods defined by a parent class work on instances of
    /// any descendant. See [`class::with_class_data`] and [`class::with_class_data_mut`].
    ///
    /// [`class::with_class_data`]: ./fn.with_class_data.html
    /// [`class::with_class_data_mut`]: ./fn.with_class_data_mut.html
    type Parent: Class;

    /// The number of classes in the native inheritance chain, from the root class up to and
    /// including `Self`. A root class has `DEPTH == 1`; [`NoParent`] has `DEPTH == 0`.
    const DEPTH: usize = <Self::Parent as Class>::DEPTH + 1;
    /// The position of this class' native data inside an instance. `INDEX == DEPTH - 1`.
    const INDEX: usize = Self::DEPTH - 1;

    /// Initializes the properties and methods of this class.
    fn init(class: &mut ClassBuilder<'_>) -> JsResult<()>;

    /// Creates the internal data for an instance of this class.
    fn data_constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<Self>;

    /// Returns the arguments forwarded to the parent class' constructor during instance
    /// construction.
    ///
    /// By default all arguments are forwarded unchanged, mirroring `super(...args)` in
    /// JavaScript. Override this method to forward a subset of the arguments.
    #[allow(unused_variables)]
    fn parent_args<'a>(
        new_target: &JsValue,
        args: &'a [JsValue],
        context: &mut Context,
    ) -> JsResult<&'a [JsValue]> {
        Ok(args)
    }

    /// Initializes the properties of the constructed object for an instance of this class.
    ///
    /// Useful to initialize additional properties for the constructed object that aren't
    /// stored inside the native data.
    #[allow(unused_variables)] // Saves work when IDEs autocomplete trait impls.
    fn object_constructor(
        instance: &JsObject,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<()> {
        Ok(())
    }

    /// Creates a new [`JsObject`] with its internal data set to the result of calling
    /// [`Class::data_constructor`] and [`Class::object_constructor`].
    ///
    /// If this class has a native parent class, the instance's native data is stored in a
    /// [`NativeClassData`] carrier holding one slot per class in the chain, and every layer is
    /// constructed in order (parent first).
    ///
    /// # Errors
    ///
    /// - Throws an error if `new_target` is undefined.
    /// - Throws an error if this class is not registered in `new_target`'s realm.
    ///   See [`Context::register_global_class`].
    ///
    /// <div class="warning">
    /// Overriding this method could be useful for certain usages, but incorrectly implementing this
    /// could lead to weird errors like missing inherited methods or incorrect internal data.
    /// </div>
    fn construct(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsObject> {
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message(format!(
                    "cannot call constructor of native class `{}` without new",
                    Self::NAME
                ))
                .into());
        }

        let prototype = 'proto: {
            let realm = if let Some(constructor) = new_target.as_object() {
                if let Some(proto) = constructor.get(PROTOTYPE, context)?.as_object() {
                    break 'proto proto.clone();
                }
                constructor.get_function_realm(context)?
            } else {
                context.realm().clone()
            };
            realm
                .get_class::<Self>()
                .ok_or_else(|| {
                    JsNativeError::typ().with_message(format!(
                        "could not find native class `{}` in the map of registered classes",
                        Self::NAME
                    ))
                })?
                .prototype()
        };

        if TypeId::of::<Self::Parent>() == TypeId::of::<NoParent>() {
            // Root class: keep the historical layout where the object data is `Self`, so
            // `downcast_ref`/`downcast_mut` and the `JsClass<T>` interop work unchanged.
            let data = Self::data_constructor(new_target, args, context)?;

            let object =
                JsObject::from_proto_and_data_with_shared_shape(context.root_shape(), prototype, data);

            Self::object_constructor(&object.upcast(), args, context)?;

            Ok(object.upcast())
        } else {
            // Subclass: instances carry a `NativeClassData` with one slot per layer, built in
            // parent-to-child order.
            let data = NativeClassData::new::<Self>();

            let object = JsObject::from_proto_and_data_with_shared_shape(
                context.root_shape(),
                prototype,
                data,
            )
            .upcast();

            build_chain::<Self>(new_target, &object, args, context)?;
            Self::object_constructor(&object, args, context)?;

            Ok(object)
        }
    }

    /// Constructs an instance of this class from its inner native data.
    ///
    /// Note that the default implementation won't call [`Class::data_constructor`], but it will
    /// call [`Class::object_constructor`] with no arguments.
    ///
    /// # Errors
    /// - Throws an error if this class is not registered in the context's realm. See
    ///   [`Context::register_global_class`].
    ///
    /// # Panics
    ///
    /// Panics if this class has a native parent class; constructing a parent-aware instance from
    /// its data requires the JavaScript `new` path.
    ///
    /// <div class="warning">
    /// Overriding this method could be useful for certain usages, but incorrectly implementing this
    /// could lead to weird errors like missing inherited methods or incorrect internal data.
    /// </div>
    fn from_data(data: Self, context: &mut Context) -> JsResult<JsObject> {
        assert!(
            TypeId::of::<Self::Parent>() == TypeId::of::<NoParent>(),
            "Class::from_data is only valid for classes without a native parent class"
        );

        let prototype = context
            .get_global_class::<Self>()
            .ok_or_else(|| {
                JsNativeError::typ().with_message(format!(
                    "could not find native class `{}` in the map of registered classes",
                    Self::NAME
                ))
            })?
            .prototype();

        let object =
            JsObject::from_proto_and_data_with_shared_shape(context.root_shape(), prototype, data);

        let object = object.upcast();
        Self::object_constructor(&object, &[], context)?;

        Ok(object)
    }
}

/// Trait object stored in each slot of a [`NativeClassData`].
pub trait ClassSlot: Any + Trace {
    /// Returns a reference to the underlying data as `dyn Any`.
    fn as_any_ref(&self) -> &dyn Any;
    /// Returns a mutable reference to the underlying data as `dyn Any`.
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Any + Trace> ClassSlot for T {
    fn as_any_ref(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// The per-instance native data store for a native class hierarchy.
///
/// One slot per class in the chain; [`Class::INDEX`] addresses a class' slot directly. The slot
/// list is sized to the leaf class' [`Class::DEPTH`] once at construction time and never resized.
#[derive(Trace, Finalize, JsData)]
pub struct NativeClassData {
    slots: Box<[GcRefCell<Option<Box<dyn ClassSlot>>>]>,
}

impl std::fmt::Debug for NativeClassData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeClassData")
            .field("slots", &self.slots.len())
            .finish()
    }
}

impl NativeClassData {
    /// Build a registry sized for a chain ending at `T` (the leaf class).
    fn new<T: Class>() -> Self {
        Self {
            slots: (0..T::DEPTH).map(|_| GcRefCell::new(None)).collect(),
        }
    }

    /// Write `T`'s data into its slot. `T::INDEX` is a compile-time constant.
    fn set_slot<T: Class>(&self, data: T) {
        *self.slots[T::INDEX].borrow_mut() = Some(Box::new(data));
    }

    /// Read a class' data through the callback.
    fn with<T: Class, R>(&self, f: impl FnOnce(&T) -> R) -> JsResult<R> {
        let slot = self.slots.get(T::INDEX).ok_or_else(|| {
            JsNativeError::typ().with_message(format!(
                "native class data index {} out of range",
                T::INDEX
            ))
        })?;

        let borrowed = slot.borrow();
        let Some(slot_ref) = borrowed.as_deref() else {
            return Err(JsNativeError::typ()
                .with_message("native class data slot not initialized")
                .into());
        };

        let data = slot_ref
            .as_any_ref()
            .downcast_ref::<T>()
            .ok_or_else(|| JsNativeError::typ().with_message("native class data type mismatch"))?;

        Ok(f(data))
    }

    /// Mutate a class' data through the callback.
    fn with_mut<T: Class, R>(&self, f: impl FnOnce(&mut T) -> R) -> JsResult<R> {
        let slot = self.slots.get(T::INDEX).ok_or_else(|| {
            JsNativeError::typ().with_message(format!(
                "native class data index {} out of range",
                T::INDEX
            ))
        })?;

        let mut borrowed = slot.borrow_mut();
        let Some(slot_ref) = borrowed.as_deref_mut() else {
            return Err(JsNativeError::typ()
                .with_message("native class data slot not initialized")
                .into());
        };

        let data = slot_ref
            .as_any_mut()
            .downcast_mut::<T>()
            .ok_or_else(|| JsNativeError::typ().with_message("native class data type mismatch"))?;

        Ok(f(data))
    }
}

/// Recursively build every layer of the hierarchy into the same final instance, parent first.
fn build_chain<T: Class>(
    new_target: &JsValue,
    instance: &JsObject,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<()> {
    if TypeId::of::<T::Parent>() != TypeId::of::<NoParent>() {
        let parent_args = T::parent_args(new_target, args, context)?;
        build_chain::<T::Parent>(new_target, instance, parent_args, context)?;
    }

    let data = T::data_constructor(new_target, args, context)?;
    set_class_data::<T>(instance, data)?;

    Ok(())
}

/// Write a class' layer data into the instance's native data carrier.
fn set_class_data<T: Class>(instance: &JsObject, data: T) -> JsResult<()> {
    let registry = instance
        .downcast_ref::<NativeClassData>()
        .ok_or_else(|| JsNativeError::typ().with_message("instance has no native class data"))?;
    registry.set_slot::<T>(data);
    Ok(())
}

/// Read a class' layer data from the instance.
///
/// This is the idiomatic way to access the native data of a class inside one of its methods: the
/// data type is fixed by the class that defined the method, not by the concrete type of `this`, so
/// a method defined by a parent class works on instances of any descendant.
///
/// On instances of a root class (no parent), the instance's data is stored directly as the
/// object data, so this also serves as a replacement for `downcast_ref` that works for the whole
/// inheritance chain.
///
/// # Errors
///
/// Returns a `TypeError` if `instance` is not a native class instance carrying `T`'s data.
pub fn with_class_data<T: Class, R>(instance: &JsObject, f: impl FnOnce(&T) -> R) -> JsResult<R> {
    if let Some(data) = instance.downcast_ref::<T>() {
        return Ok(f(&*data));
    }

    let registry = instance
        .downcast_ref::<NativeClassData>()
        .ok_or_else(|| JsNativeError::typ().with_message("instance has no native class data"))?;
    registry.with::<T, R>(f)
}

/// Mutate a class' layer data on the instance.
///
/// # Errors
///
/// Returns a `TypeError` if `instance` is not a native class instance carrying `T`'s data.
pub fn with_class_data_mut<T: Class, R>(
    instance: &JsObject,
    f: impl FnOnce(&mut T) -> R,
) -> JsResult<R> {
    if let Some(mut data) = instance.downcast_mut::<T>() {
        return Ok(f(&mut data));
    }

    let registry = instance
        .downcast_ref::<NativeClassData>()
        .ok_or_else(|| JsNativeError::typ().with_message("instance has no native class data"))?;
    registry.with_mut::<T, R>(f)
}

/// Class builder which allows adding methods and static methods to the class.
#[derive(Debug)]
pub struct ClassBuilder<'ctx> {
    builder: ConstructorBuilder<'ctx>,
}

impl<'ctx> ClassBuilder<'ctx> {
    /// Create a new `ClassBuilder` from a [`Class`] type.
    pub fn new<T>(context: &'ctx mut Context) -> Self
    where
        T: Class,
    {
        let (inherit, custom_prototype) = if TypeId::of::<T::Parent>() != TypeId::of::<NoParent>() {
            // Wire the native inheritance chain: the class' prototype inherits from the parent's
            // prototype and the class' constructor inherits from the parent's constructor.
            let parent = context
                .get_global_class::<T::Parent>()
                .expect("parent class must be registered before its children");
            (Some(parent.prototype()), Some(parent.constructor()))
        } else {
            (None, None)
        };

        let mut builder = ConstructorBuilder::new(
            context,
            NativeFunction::from_fn_ptr(|t, a, c| T::construct(t, a, c).map(JsValue::from)),
        );
        builder.name(T::NAME);
        builder.length(T::LENGTH);

        if let Some(prototype) = inherit {
            builder.inherit(prototype);
        }
        if let Some(constructor) = custom_prototype {
            builder.custom_prototype(constructor);
        }

        Self { builder }
    }

    /// Create the [`StandardConstructor`] from this class builder.
    #[must_use]
    pub fn build(self) -> StandardConstructor {
        self.builder.build()
    }

    /// Add a method to the class.
    ///
    /// It is added to `prototype`.
    pub fn method<N>(&mut self, name: N, length: usize, function: NativeFunction) -> &mut Self
    where
        N: Into<FunctionBinding>,
    {
        self.builder.method(function, name, length);
        self
    }

    /// Add a static method to the class.
    ///
    /// It is added to class object itself.
    pub fn static_method<N>(
        &mut self,
        name: N,
        length: usize,
        function: NativeFunction,
    ) -> &mut Self
    where
        N: Into<FunctionBinding>,
    {
        self.builder.static_method(function, name, length);
        self
    }

    /// Add a data property to the class, with the specified attribute.
    ///
    /// It is added to `prototype`.
    pub fn property<K, V>(&mut self, key: K, value: V, attribute: Attribute) -> &mut Self
    where
        K: Into<PropertyKey>,
        V: Into<JsValue>,
    {
        self.builder.property(key, value, attribute);
        self
    }

    /// Add a static data property to the class, with the specified attribute.
    ///
    /// It is added to class object itself.
    pub fn static_property<K, V>(&mut self, key: K, value: V, attribute: Attribute) -> &mut Self
    where
        K: Into<PropertyKey>,
        V: Into<JsValue>,
    {
        self.builder.static_property(key, value, attribute);
        self
    }

    /// Add an accessor property to the class, with the specified attribute.
    ///
    /// It is added to `prototype`.
    pub fn accessor<K>(
        &mut self,
        key: K,
        get: Option<JsFunction>,
        set: Option<JsFunction>,
        attribute: Attribute,
    ) -> &mut Self
    where
        K: Into<PropertyKey>,
    {
        self.builder.accessor(key, get, set, attribute);
        self
    }

    /// Add a static accessor property to the class, with the specified attribute.
    ///
    /// It is added to class object itself.
    pub fn static_accessor<K>(
        &mut self,
        key: K,
        get: Option<JsFunction>,
        set: Option<JsFunction>,
        attribute: Attribute,
    ) -> &mut Self
    where
        K: Into<PropertyKey>,
    {
        self.builder.static_accessor(key, get, set, attribute);
        self
    }

    /// Add a property descriptor to the class, with the specified attribute.
    ///
    /// It is added to `prototype`.
    pub fn property_descriptor<K, P>(&mut self, key: K, property: P) -> &mut Self
    where
        K: Into<PropertyKey>,
        P: Into<CompletePropertyDescriptor>,
    {
        self.builder.property_descriptor(key, property);
        self
    }

    /// Add a static property descriptor to the class, with the specified attribute.
    ///
    /// It is added to class object itself.
    pub fn static_property_descriptor<K, P>(&mut self, key: K, property: P) -> &mut Self
    where
        K: Into<PropertyKey>,
        P: Into<CompletePropertyDescriptor>,
    {
        self.builder.static_property_descriptor(key, property);
        self
    }

    /// Return the current context.
    #[inline]
    pub fn context(&mut self) -> &mut Context {
        self.builder.context()
    }
}