pub enum HandleResult<T> {
    UnHandle,
    Handled(T),
}

impl<T: Default> std::default::Default for HandleResult<T> {
    fn default() -> Self {
        Self::UnHandle
    }
}

impl<T> HandleResult<T> {
    pub fn handle(&mut self, value: T) {
        if let HandleResult::UnHandle = self {
            let _ = std::mem::replace(self, HandleResult::Handled(value));
        }
    }

    pub fn is_handled(&self) -> bool {
        if let Self::UnHandle = self {
            return false;
        }

        true
    }

    pub fn is_unhandle(&self) -> bool {
        !self.is_handled()
    }

    pub fn value(self) -> Option<T> {
        match self {
            HandleResult::UnHandle => None,
            HandleResult::Handled(value) => Some(value),
        }
    }

    pub fn unwrap_or(self, default: T) -> T {
        match self {
            HandleResult::UnHandle => default,
            HandleResult::Handled(value) => value,
        }
    }

    pub fn unwrap_or_with<FN>(self, default: FN) -> T
    where
        FN: FnOnce() -> T,
    {
        match self {
            HandleResult::UnHandle => default(),
            HandleResult::Handled(value) => value,
        }
    }
}
