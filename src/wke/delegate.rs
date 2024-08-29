#[allow(unused_macros)]
macro_rules! DefineMulticastDelegateImpl {
    ($name:ident, ($($param_name:ident: $param_type:ty),*), $traits: expr) => {
        #[derive(Default)]
        pub struct $name {
            callbacks: Vec<Box<dyn Fn($($param_type),*) -> anyhow::Result<()> + $traits>>
        }

        impl $name {
            pub fn bind<FN>(&mut self, func: FN) -> &mut Self
            where
                FN: Fn($($param_type), *) -> anyhow::Result<()> + $traits,
            {
                self.callbacks.push(Box::new(func));
                return self;
            }

            pub fn emit(&self, $($param_name: $param_type),*) -> anyhow::Result<()>
            {
                if self.callbacks.len() == 1 {
                    if let Some(callback) = self.callbacks.get(0) {
                        return callback($($param_name), *);
                    }

                    return anyhow::Ok(());
                }

                for callback in self.callbacks.iter() {
                    callback($($param_name.clone()), *)?;
                }
                return anyhow::Ok(());
            }
        }
    };
}


#[macro_export]
macro_rules! DefineMulticastDelegate {
    ($name:ident, ($($param_name:ident: $param_type:ty),*)) => {
        #[derive(Default)]
        pub struct $name {
            callbacks: Vec<Box<dyn Fn($($param_type),*) ->  anyhow::Result<()> + Send + Sync + 'static>>
        }

        impl $name {
            pub fn bind<FN>(&mut self, func: FN) -> &mut Self
            where
                FN: Fn($($param_type), *) -> anyhow::Result<()> + Send + Sync + 'static,
            {
                self.callbacks.push(Box::new(func));
                return self;
            }

            pub fn emit(&self, $($param_name: $param_type),*) -> anyhow::Result<()>
            {
                if self.callbacks.len() == 1 {
                    if let Some(callback) = self.callbacks.get(0) {
                        return callback($($param_name), *);
                    }

                    return anyhow::Ok(());
                }

                for callback in self.callbacks.iter() {
                    callback($($param_name.clone()), *)?;
                }
                return anyhow::Ok(());
            }
        }
    };
}
