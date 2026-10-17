#' Classes sharing an author-defined topic
#'
#' Members from secondary Rust impl blocks and trait impls belong to this topic too.
#' @name impl_topics
#' @param value Initial value, documented by the topic.
#' @param offset Offset, documented by the topic.
#' @param amounts Numeric amounts, documented by the topic.
#' @param x,.mx_self,self An object, documented by the topic.
#' @param name A member name, documented by the topic.
#' @param y,to A vector prototype, documented by the topic.
#' @param ... Additional arguments, documented by the topic.
#' @param .ptr Internal pointer, documented by the topic.
#' @param peer A `TopicS7Peer` object, documented by the topic.
NULL

#' An explicit member topic
#' @name impl_topic_override
#' @seealso [impl_topics] for the class topic these members override.
#' @param x,.mx_self An object.
#' @param ... Additional arguments.
NULL
