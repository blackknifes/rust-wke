pub trait IntoDelegateParam<'a, DEST> {
    fn into_param(&'a self) -> DEST;
}

impl<'a, DEST> IntoDelegateParam<'a, DEST> for DEST
where
    DEST: Clone,
{
    fn into_param(&'a self) -> DEST {
        self.clone()
    }
}

impl<'life0, 'life1, DEST> IntoDelegateParam<'life0, &'life1 DEST> for DEST
where
    'life0: 'life1,
{
    fn into_param(&'life0 self) -> &'life1 DEST {
        self
    }
}

pub trait IntoDelegateParamMut<'a, DEST> {
    fn into_param(&'a mut self) -> DEST;
}

impl<'life0, 'life1, DEST> IntoDelegateParamMut<'life0, &'life1 mut DEST> for DEST
where
    'life0: 'life1,
{
    fn into_param(&'life0 mut self) -> &'life1 mut DEST {
        self
    }
}

#[macro_export]
macro_rules! DefineMulticastDelegate {
    //同步委托
    ($name:ident, ($($param_name:ident: $param_type:ty),*) -> () $(, $trait1:ident $(+ $traits:ident)*)? ) => {
        DefineMulticastDelegate!($name, ($($param_name: $param_type),*) $(, $trait1 $(+ $traits)*)?);
    };

    //同步委托
    ($name:ident, ($($param_name:ident: $param_type:ty),*) $(,$trait1:ident $(+ $traits:ident)*)? ) => {
        #[derive(Default)]
        pub struct $name {
            sequence: usize,
            callbacks: std::collections::BTreeMap<usize, Box<
                dyn Fn($($param_type),*) ->
                    $crate::error::Result<()> + 'static $(+ $trait1 $(+ $traits)*)?
            >>
        }

        impl $name {
            pub fn add<FN>(&mut self, func: FN) -> usize
            where
                FN: Fn($($param_type), *) ->
                    $crate::error::Result<()> + 'static $(+ $trait1 $(+ $traits)*)?,
            {
                self.sequence = self.sequence + 1;
                let id = self.sequence;
                self.callbacks.insert(id, Box::new(func));
                return id;
            }

            pub fn remove(&mut self, id: usize)
            {
                self.callbacks.remove(&id);
            }

            pub fn emit(&self, $($param_name: $param_type),*)
            {
                #[allow(unused_imports)]
                use $crate::delegate::IntoDelegateParam;
                #[allow(unused_imports)]
                use $crate::delegate::IntoDelegateParamMut;

                for callback in self.callbacks.values() {
                    if let Err(err) = callback($($param_name.into_param()), *) {
                        log::error!("emit failed: {}", err);
                    }
                }
            }
        }
    };


    //异步委托
    ($name:ident, async ($($param_name:ident: $param_type:ty),*) -> () $(, $trait1:ident $(+ $traits:ident)*)? ) => {
        DefineMulticastDelegate!($name, async ($($param_name: $param_type),*) $(, $trait1 $(+ $traits)*)?);
    };
    //异步委托
    ($name:ident, async ($($param_name:ident: $param_type:ty),*) $(,$trait1:ident $(+ $traits:ident)*)? ) => {
        #[derive(Default)]
        pub struct $name {
            sequence: usize,
            callbacks: std::collections::BTreeMap<usize,
                Box<dyn Fn($($param_type),*) ->
                    std::pin::Pin<Box<
                        dyn std::future::Future<Output = $crate::error::Result<()>> + 'static $(+ $trait1 $(+ $traits)*)?
                    >> + 'static $(+ $trait1 $(+ $traits)*)?
                >
            >
        }

        impl $name {
            pub fn add<FN, FUT>(&mut self, func: FN) -> usize
            where
                FUT: std::future::Future<Output = $crate::error::Result<()>> + 'static $(+ $trait1 $(+ $traits)*)?,
                FN: Fn($($param_type), *) -> FUT + 'static $(+ $trait1 $(+ $traits)*)?,
            {
                self.sequence = self.sequence + 1;
                let id = self.sequence;
                self.callbacks.insert(id, Box::new(move |$($param_name)*| {
                    let fut = func($($param_name)*);
                    Box::pin(fut)
                }));
                return id;
            }

            pub fn remove(&mut self, id: usize)
            {
                self.callbacks.remove(&id);
            }

            pub async fn emit(&self, $($param_name: $param_type),*)
            {
                #[allow(unused_imports)]
                use $crate::delegate::IntoDelegateParam;
                #[allow(unused_imports)]
                use $crate::delegate::IntoDelegateParamMut;

                for callback in self.callbacks.values() {
                    if let Err(err) = callback($($param_name.into_param()), *).await {
                        log::error!("emit failed: {}", err);
                    }
                }
            }
        }
    };
}
