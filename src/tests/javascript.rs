use crate::{
    error::{Error, Result},
    javascript::{FromJs, IntoJs, JsDelegate, JsValue},
};

pub struct JsFunction<RET: IntoJs>(Box<dyn Fn(&[&JsValue]) -> RET>);

impl<RET: IntoJs> JsDelegate for JsFunction<RET> {
    fn has_call(&self) -> bool {
        true
    }

    fn call(&mut self, args: &[&JsValue]) -> Result<JsValue> {
        self.0(args).into_js()
    }
}

impl<FN, RET> From<FN> for JsFunction<RET>
where
    RET: IntoJs,
    FN: Fn(&[&JsValue]) -> RET + 'static,
{
    fn from(value: FN) -> Self {
        Self(Box::new(value))
    }
}

pub struct TestGetterSetter {
    number: i32,
    string: String,
    const_value: String,
}
impl std::default::Default for TestGetterSetter {
    fn default() -> Self {
        Self {
            number: Default::default(),
            string: Default::default(),
            const_value: "const_value test".to_owned(),
        }
    }
}

impl JsDelegate for TestGetterSetter {
    fn has_get(&self) -> bool {
        true
    }

    fn has_set(&self) -> bool {
        true
    }

    fn has_call(&self) -> bool {
        false
    }

    fn get(&mut self, name: &str) -> Result<JsValue> {
        match name {
            "number" => self.number.into_js(),
            "string" => self.string.into_js(),
            "const_value" => self.const_value.into_js(),
            _ => JsValue::undefined(),
        }
    }

    fn set(&mut self, name: &str, val: &JsValue) -> Result<()> {
        match name {
            "number" => self.number = FromJs::from_js(val)?,
            "string" => self.string = FromJs::from_js(val)?,
            _ => return Err(Error::NotImplement),
        };
        Ok(())
    }

    fn call(&mut self, _args: &[&JsValue]) -> Result<JsValue> {
        Err(Error::NotImplement)
    }

    fn finalize(&mut self) -> Result<()> {
        println!("TestGetterSetter::finalize");
        Ok(())
    }
}
