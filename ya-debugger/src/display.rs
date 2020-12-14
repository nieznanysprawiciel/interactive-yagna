use std::fmt::{Error, Formatter};
use ya_client_model::activity::StatePair;

pub struct DisplayEnabler<'a, Type>(pub &'a Type);

pub trait EnableDisplay<Type> {
    fn display(&self) -> DisplayEnabler<Type>;
}

impl<Type> EnableDisplay<Type> for Type {
    fn display(&self) -> DisplayEnabler<Type> {
        DisplayEnabler(self)
    }
}

impl<'a> std::fmt::Display for DisplayEnabler<'a, StatePair> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self.0 {
            StatePair(state, None) => write!(f, "{:?}", state),
            StatePair(state, Some(transition)) => {
                write!(f, "{:?} -> {:?}", state, transition)
            }
        }
    }
}

impl<'a, Type> std::fmt::Display for DisplayEnabler<'a, Option<Type>>
where
    Type: std::fmt::Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match &self.0 {
            Some(id) => id.fmt(f),
            // TODO: Someone funny could set appSessionId to "None" string.
            None => write!(f, "None"),
        }
    }
}

impl<'a, Type> std::fmt::Display for DisplayEnabler<'a, Vec<Type>>
where
    Type: std::fmt::Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        for element in self.0 {
            write!(f, "{} ", element)?;
        }
        Ok(())
    }
}
