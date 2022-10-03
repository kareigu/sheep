use serenity::{
  Error,
  prelude::Context, 
  model::prelude::interaction::application_command::ApplicationCommandInteraction
};


pub async fn text_response<D>(
  ctx: &Context, 
  command: ApplicationCommandInteraction, text: D) -> Result<(), Error>
where D: ToString, {
  let _a = command
    .edit_original_interaction_response(&ctx.http, |response| {
      response
        .embed(|embed| {
          embed
            .title(text)
            .colour(0xFFFFFF)
        })
    }).await?;
  Ok(())
}