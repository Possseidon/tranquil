use std::{fmt, marker::PhantomData};

use crate::snowflake::Snowflake;

struct User;
struct Channel;
struct Role;

pub struct Id<T> {
    pub snowflake: Snowflake,
    kind: PhantomData<T>,
}

impl fmt::Display for Id<User> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<@{}>", self.snowflake)
    }
}

impl fmt::Display for Id<Channel> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<#{}>", self.snowflake)
    }
}

impl fmt::Display for Id<Role> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<@&{}>", self.snowflake)
    }
}
