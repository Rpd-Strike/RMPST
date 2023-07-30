use crate::rollpi::syntax::TagKey;

#[derive(Default)]
pub struct TagCreator
{
    tag_count: usize,
}

impl TagCreator 
{
    pub fn create_new_tag(self: &mut Self) -> TagKey
    {
        let tag = TagKey(format!("_t_p_{}", self.tag_count));
        self.tag_count += 1;
        tag
    }
}