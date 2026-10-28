use kvas_engine::prelude::*;

script_params! {
    /// Prints the tags of the object once at start.
    pub struct LogTags {
        done: Local<bool> = Local(false),
    }
}

impl Script for LogTags {
    fn update(&mut self, ctx: &mut Ctx) {
        if !*self.done {
            *self.done = true;
            let tags = ctx.tags_of(ctx.entity);
            ctx.log(format!("tags: {tags:?}"));
        }
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
