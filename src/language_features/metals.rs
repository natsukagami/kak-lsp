use indoc::formatdoc;
use lsp_types::{
    notification::Notification, InlayHint, InlayHintLabel, InlayHintTooltip, MarkupContent, Range,
    Url,
};

use crate::{
    context::Context,
    types::{EditorMeta, ServerName},
    util::{editor_quote, short_file_path},
};

use super::inlay_hints::inlay_hints_to_ranges;

// DECORATION PROTOCOL

pub struct PublishDecorations {}

impl Notification for PublishDecorations {
    type Params = PublishDecorationsParams;
    const METHOD: &'static str = "metals/publishDecorations";
}

/// Handle `metals/publishDecorations`.
pub fn publish_decorations(meta: EditorMeta, params: PublishDecorationsParams, ctx: &mut Context) {
    let server_name: ServerName = meta.server.clone().unwrap_or_else(|| "metals".to_owned());
    let buffile = params
        .uri
        .to_file_path()
        .expect("Must be a file path")
        .to_str()
        .expect("Not UTF-8 path")
        .to_owned();
    let buffile_rel = {
        let root_path = &ctx
            .language_servers
            .get(&server_name[..])
            .expect("has server ;(")
            .root_path;
        short_file_path(buffile.as_ref(), root_path)
    };
    // println!(
    //     "buffile = {}, documents = {:?}",
    //     buffile,
    //     ctx.documents.iter().map(|(k, _)| k).collect::<Vec<_>>()
    // );
    // find the file
    let document = match ctx.documents.get(&buffile) {
        Some(d) => d,
        None => return,
    };

    // convert changes to InlayHints
    let inlay_hints = params.options.into_iter().filter_map(|v| {
        let render = v.render_options.after?;
        Some((
            server_name.clone(),
            InlayHint {
                position: v.range.end,
                label: InlayHintLabel::String(render.content_text),
                kind: None,
                text_edits: None,
                tooltip: v.hover_message.map(InlayHintTooltip::MarkupContent),
                padding_left: None,
                padding_right: None,
                data: None,
            },
        ))
    });

    let ranges = inlay_hints_to_ranges(document, inlay_hints, ctx);

    let command = formatdoc!("set-option buffer lsp_inlay_hints %val{{timestamp}} {ranges}");
    let command = format!(
        "evaluate-commands -buffer {} -- {}",
        editor_quote(&buffile_rel),
        editor_quote(&command)
    );
    ctx.exec(meta, command)
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishDecorationsParams {
    pub uri: Url,
    pub options: Vec<DecorationOptions>,
    #[serde(default)]
    pub inline: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DecorationOptions {
    pub range: Range,
    pub hover_message: Option<MarkupContent>,
    pub render_options: ThemableDecorationInstanceRenderOption,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemableDecorationInstanceRenderOption {
    pub after: Option<ThemableDecorationAttachmentRenderOptions>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemableDecorationAttachmentRenderOptions {
    pub content_text: String,
    pub color: String,      // for now, always "green"
    pub font_style: String, // for now, always "italics"
}
