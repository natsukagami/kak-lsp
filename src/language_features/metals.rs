use lsp_types::{
    notification::Notification, InlayHint, InlayHintLabel, InlayHintTooltip, MarkupContent, Range,
    Url,
};

use crate::{context::Context, types::EditorMeta, util::editor_quote, ServerId};

use super::inlay_hints::inlay_hints_to_ranges;

// DECORATION PROTOCOL

#[allow(dead_code)]
pub enum PublishDecorations {}

impl Notification for PublishDecorations {
    type Params = PublishDecorationsParams;
    const METHOD: &'static str = "metals/publishDecorations";
}

/// Handle `metals/publishDecorations`.
pub fn publish_decorations(
    meta: EditorMeta,
    server_id: ServerId,
    params: PublishDecorationsParams,
    ctx: &mut Context,
) {
    const METALS_DECORATION_OPTION: &str = "lsp_metals_decoration";
    let Some(document) = ctx.documents.get(&meta.buffile) else {
        return;
    };
    // convert changes to InlayHints
    let inlay_hints = params.options.into_iter().filter_map(|v| {
        let render = v.render_options.after?;
        Some((
            server_id,
            InlayHint {
                position: v.range.end,
                label: InlayHintLabel::String(render.content_text),
                kind: None,
                text_edits: None,
                tooltip: v.hover_message.map(InlayHintTooltip::MarkupContent),
                padding_left: Some(true),
                padding_right: None,
                data: None,
            },
        ))
    });
    let ranges = inlay_hints_to_ranges(document, inlay_hints, ctx);
    let version = meta.version;
    let command = format!("set-option buffer {METALS_DECORATION_OPTION} {version} {ranges}");
    let command = format!(
        "evaluate-commands -buffer {} -- {}",
        editor_quote(&meta.buffile),
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
