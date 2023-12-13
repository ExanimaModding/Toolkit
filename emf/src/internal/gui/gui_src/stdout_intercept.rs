macro_rules! def_special {
    ($viz:vis $id:ident : $ty:ty) => {
        #[allow(non_snake_case)]
        $viz mod $id {
            use std::ops::{Deref,DerefMut};
            type Ctx = $ty;

            use std::cell::Cell;
            thread_local! {
                static VAR: Cell<Option<*mut Ctx>> = Cell::new(None);
            }

            pub fn borrow()->Borrow {
                Borrow { cur: VAR.with(|x| x.take()).unwrap() }
            }

            pub fn lend<F,O>(ctx:&mut Ctx, f:F)->O where F:FnOnce()->O {
                struct Lend {
                    ctx: *mut Ctx,
                    prev: Option<*mut Ctx>
                }

                let ctx = ctx as *mut Ctx;
                let prev = VAR.with(|x| x.replace(Some(ctx)));

                let _lent = Lend { ctx, prev };

                impl Drop for Lend {
                    fn drop(&mut self) {
                        assert_eq!(VAR.with(|x| x.replace(self.prev)), Some(self.ctx));
                    }
                }

                f()
            }

            pub struct Borrow {
                cur: *mut Ctx,
            }

            impl Deref for Borrow {
                type Target = Ctx;
                fn deref(&self)->&Ctx {
                    unsafe { &* self.cur }
                }
            }

            impl DerefMut for Borrow {
                fn deref_mut(&mut self)->&mut Ctx {
                    unsafe { &mut * self.cur }
                }
            }

            impl Drop for Borrow {
                fn drop(&mut self) {
                    assert!(VAR.with(|x| x.replace(Some(self.cur))).is_none());
                }
            }
        }
    }
}

pub(crate) use def_special;
