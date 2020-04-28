#![feature(prelude_import)]
#![allow(clippy::blacklisted_name)]
#[prelude_import]
use std::prelude::v1::*;
#[macro_use]
extern crate std;
use thespian::Actor;
pub struct MyActor {
    id: usize,
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::fmt::Debug for MyActor {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match *self {
            MyActor { id: ref __self_0_0 } => {
                let mut debug_trait_builder = f.debug_struct("MyActor");
                let _ = debug_trait_builder.field("id", &&(*__self_0_0));
                debug_trait_builder.finish()
            }
        }
    }
}
impl thespian::Actor for MyActor {
    type Proxy = MyActorProxy;
}
pub struct MyActorProxy {
    inner: thespian::ProxyFor<MyActor>,
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::fmt::Debug for MyActorProxy {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match *self {
            MyActorProxy {
                inner: ref __self_0_0,
            } => {
                let mut debug_trait_builder = f.debug_struct("MyActorProxy");
                let _ = debug_trait_builder.field("inner", &&(*__self_0_0));
                debug_trait_builder.finish()
            }
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::clone::Clone for MyActorProxy {
    #[inline]
    fn clone(&self) -> MyActorProxy {
        match *self {
            MyActorProxy {
                inner: ref __self_0_0,
            } => MyActorProxy {
                inner: ::core::clone::Clone::clone(&(*__self_0_0)),
            },
        }
    }
}
impl thespian::ActorProxy for MyActorProxy {
    type Actor = MyActor;
    fn new(inner: thespian::ProxyFor<MyActor>) -> Self {
        Self { inner }
    }
}
impl MyActor {
    pub fn multiple_params(&self, _first: usize, _second: String) {
        {
            ::std::rt::begin_panic("not implemented")
        }
    }
}
impl MyActorProxy {
    pub fn multiple_params(
        &mut self,
        _first: usize,
        _second: String,
    ) -> thespian::Result<impl std::future::Future<Output = ()>> {
        self.inner
            .send_message(MyActor__multiple_params(_first, _second))
    }
}
#[doc(hidden)]
#[allow(bad_style)]
pub struct MyActor__multiple_params(usize, String);
impl thespian::Message for MyActor__multiple_params {
    type Actor = MyActor;
    type Output = ();
    fn handle(
        self,
        actor: &mut Self::Actor,
    ) -> thespian::futures::future::BoxFuture<'_, Self::Output> {
        thespian::futures::future::FutureExt::boxed(
            async move { actor.multiple_params(self.0, self.1) },
        )
    }
}
#[main]
pub fn main() -> () {
    extern crate test;
    test::test_main_static(&[])
}
