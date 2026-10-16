//! Impl topics must include members from every block, in every class system.
use miniextendr_api::{ExternalPtr, miniextendr};

#[derive(ExternalPtr)]
pub struct TopicENV {
    value: i32,
}

/// An env class on an author topic.
/// @rdname impl_topics
#[miniextendr(env)]
impl TopicENV {
    pub fn new(value: i32) -> Self {
        Self { value }
    }
}

#[miniextendr(env)]
impl TopicENV {
    /// Read the value.
    pub fn topic_env_value(&self) -> i32 {
        self.value
    }

    /// Start from an offset.
    pub fn origin(offset: i32) -> i32 {
        offset
    }
}

#[derive(ExternalPtr)]
pub struct TopicR6 {
    value: i32,
}

/// An R6 class on an author topic.
/// @rdname impl_topics
#[miniextendr(r6)]
impl TopicR6 {
    /// @param value Initial value for the R6 instance.
    pub fn new(value: i32) -> Self {
        Self { value }
    }
}

#[miniextendr(r6)]
impl TopicR6 {
    /// Read the value.
    /// @param offset Offset added to the instance value.
    pub fn topic_r6_value(&self, offset: i32) -> i32 {
        self.value + offset
    }

    /// Start from an offset.
    pub fn origin(offset: i32) -> i32 {
        offset
    }
}

#[derive(ExternalPtr)]
pub struct TopicS3 {
    value: i32,
}

/// An S3 class on an author topic.
/// @rdname impl_topics
#[miniextendr(s3)]
impl TopicS3 {
    pub fn new(value: i32) -> Self {
        Self { value }
    }
}

#[miniextendr(s3)]
impl TopicS3 {
    /// Read the value.
    pub fn topic_s3_value(&self) -> i32 {
        self.value
    }

    /// Start from an offset.
    pub fn origin(offset: i32) -> i32 {
        offset
    }
}

#[derive(ExternalPtr)]
pub struct TopicS4 {
    value: i32,
}

/// An S4 class on an author topic.
/// @rdname impl_topics
#[miniextendr(s4)]
impl TopicS4 {
    pub fn new(value: i32) -> Self {
        Self { value }
    }
}

#[miniextendr(s4)]
impl TopicS4 {
    /// Read the value.
    pub fn topic_s4_value(&self) -> i32 {
        self.value
    }

    /// Start from an offset.
    pub fn origin(offset: i32) -> i32 {
        offset
    }
}

#[derive(ExternalPtr)]
pub struct TopicS7 {
    value: i32,
}

/// An S7 class on an author topic.
/// @rdname impl_topics
#[miniextendr(s7)]
impl TopicS7 {
    pub fn new(value: i32) -> Self {
        Self { value }
    }
}

#[miniextendr(s7)]
impl TopicS7 {
    /// Read the value.
    pub fn topic_s7_value(&self) -> i32 {
        self.value
    }

    /// Start from an offset.
    pub fn origin(offset: i32) -> i32 {
        offset
    }
}

pub struct TopicVctrs;

/// A vctrs class on an author topic.
/// @rdname impl_topics
#[miniextendr(vctrs(kind = "vctr", base = "double", abbr = "topic"))]
impl TopicVctrs {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(amounts: Vec<f64>) -> Vec<f64> {
        amounts
    }
}

#[miniextendr(vctrs(kind = "vctr", base = "double", abbr = "topic"))]
impl TopicVctrs {
    /// Start from an offset.
    pub fn origin(offset: i32) -> i32 {
        offset
    }
}

#[miniextendr(s3)]
impl TopicS3 {
    /// An explicit method topic wins over the impl topic.
    /// @rdname impl_topic_override
    pub fn topic_override(&self) -> i32 {
        self.value
    }
}
