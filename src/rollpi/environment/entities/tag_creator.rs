use crate::rollpi::syntax::TagKey;

pub struct TagCreator
{
    tag_count: usize,
    tag_prefix: String,
}

impl TagCreator 
{
    pub fn new(tag_prefix: String) -> Self
    {
        TagCreator {
            tag_count: 0,
            tag_prefix,
        }
    }

    pub fn create_new_tag(self: &mut Self) -> TagKey
    {
        let tag = TagKey(format!("_tag_{}_{}", self.tag_prefix, self.tag_count));
        self.tag_count += 1;
        tag
    }
}