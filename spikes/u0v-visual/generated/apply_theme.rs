// AUTO-GENERATED — runtime theme apply from theme.tokens
pub fn apply_resolved(ui: &crate::MainWindow, theme: &crate::token_gen::ThemeTokens) {
    let g = crate::Theme::get(ui);
    use crate::token_gen::ResolvedToken;
    if let Some(brush) = crate::color_brush_from_token(theme, "color.accent.focus") {
        g.set_color_accent_focus(brush);
    }
    if let Some(brush) = crate::color_brush_from_token(theme, "color.accent.on-accent") {
        g.set_color_accent_on_accent(brush);
    }
    if let Some(brush) = crate::color_brush_from_token(theme, "color.accent.selection") {
        g.set_color_accent_selection(brush);
    }
    if let Some(brush) = crate::color_brush_from_token(theme, "color.clip.border") {
        g.set_color_clip_border(brush);
    }
    if let Some(brush) = crate::color_brush_from_token(theme, "color.clip.ink") {
        g.set_color_clip_ink(brush);
    }
    if let Some(brush) = crate::color_brush_from_token(theme, "color.content.disabled") {
        g.set_color_content_disabled(brush);
    }
    if let Some(brush) = crate::color_brush_from_token(theme, "color.content.muted") {
        g.set_color_content_muted(brush);
    }
    if let Some(brush) = crate::color_brush_from_token(theme, "color.content.primary") {
        g.set_color_content_primary(brush);
    }
    if let Some(brush) = crate::color_brush_from_token(theme, "color.content.secondary") {
        g.set_color_content_secondary(brush);
    }
    if let Some(brush) = crate::color_brush_from_token(theme, "color.domain.path") {
        g.set_color_domain_path(brush);
    }
    if let Some(brush) = crate::color_brush_from_token(theme, "color.domain.pixel") {
        g.set_color_domain_pixel(brush);
    }
    if let Some(brush) = crate::color_brush_from_token(theme, "color.feedback.error") {
        g.set_color_feedback_error(brush);
    }
    if let Some(brush) = crate::color_brush_from_token(theme, "color.feedback.warning") {
        g.set_color_feedback_warning(brush);
    }
    if let Some(brush) = crate::color_brush_from_token(theme, "color.hover") {
        g.set_color_hover(brush);
    }
    if let Some(brush) = crate::color_brush_from_token(theme, "color.item.audio") {
        g.set_color_item_audio(brush);
    }
    if let Some(brush) = crate::color_brush_from_token(theme, "color.item.group") {
        g.set_color_item_group(brush);
    }
    if let Some(brush) = crate::color_brush_from_token(theme, "color.item.image") {
        g.set_color_item_image(brush);
    }
    if let Some(brush) = crate::color_brush_from_token(theme, "color.item.mesh") {
        g.set_color_item_mesh(brush);
    }
    if let Some(brush) = crate::color_brush_from_token(theme, "color.item.particle") {
        g.set_color_item_particle(brush);
    }
    if let Some(brush) = crate::color_brush_from_token(theme, "color.item.shape") {
        g.set_color_item_shape(brush);
    }
    if let Some(brush) = crate::color_brush_from_token(theme, "color.item.text") {
        g.set_color_item_text(brush);
    }
    if let Some(brush) = crate::color_brush_from_token(theme, "color.item.video") {
        g.set_color_item_video(brush);
    }
    if let Some(brush) = crate::color_brush_from_token(theme, "color.playhead") {
        g.set_color_playhead(brush);
    }
    if let Some(brush) = crate::color_brush_from_token(theme, "color.state.active") {
        g.set_color_state_active(brush);
    }
    if let Some(brush) = crate::color_brush_from_token(theme, "color.state.cached") {
        g.set_color_state_cached(brush);
    }
    if let Some(brush) = crate::color_brush_from_token(theme, "color.state.keyframe") {
        g.set_color_state_keyframe(brush);
    }
    if let Some(brush) = crate::color_brush_from_token(theme, "color.state.linked") {
        g.set_color_state_linked(brush);
    }
    if let Some(brush) = crate::color_brush_from_token(theme, "color.state.music") {
        g.set_color_state_music(brush);
    }
    if let Some(brush) = crate::color_brush_from_token(theme, "color.state.mute") {
        g.set_color_state_mute(brush);
    }
    if let Some(brush) = crate::color_brush_from_token(theme, "color.state.on-active") {
        g.set_color_state_on_active(brush);
    }
    if let Some(brush) = crate::color_brush_from_token(theme, "color.state.solo") {
        g.set_color_state_solo(brush);
    }
    if let Some(brush) = crate::color_brush_from_token(theme, "color.stroke.grid-major") {
        g.set_color_stroke_grid_major(brush);
    }
    if let Some(brush) = crate::color_brush_from_token(theme, "color.stroke.grid-minor") {
        g.set_color_stroke_grid_minor(brush);
    }
    if let Some(brush) = crate::color_brush_from_token(theme, "color.stroke.line") {
        g.set_color_stroke_line(brush);
    }
    if let Some(brush) = crate::color_brush_from_token(theme, "color.stroke.line-strong") {
        g.set_color_stroke_line_strong(brush);
    }
    if let Some(brush) = crate::color_brush_from_token(theme, "color.surface.bg") {
        g.set_color_surface_bg(brush);
    }
    if let Some(brush) = crate::color_brush_from_token(theme, "color.surface.inset") {
        g.set_color_surface_inset(brush);
    }
    if let Some(brush) = crate::color_brush_from_token(theme, "color.surface.overlay") {
        g.set_color_surface_overlay(brush);
    }
    if let Some(brush) = crate::color_brush_from_token(theme, "color.surface.panel") {
        g.set_color_surface_panel(brush);
    }
    if let Some(brush) = crate::color_brush_from_token(theme, "color.surface.panel-raised") {
        g.set_color_surface_panel_raised(brush);
    }
    if let Some(brush) = crate::color_brush_from_token(theme, "color.surface.vp-surround") {
        g.set_color_surface_vp_surround(brush);
    }
    if let Some(brush) = crate::color_brush_from_token(theme, "color.wave.primary") {
        g.set_color_wave_primary(brush);
    }
    if let Some(brush) = crate::color_brush_from_token(theme, "color.wave.secondary") {
        g.set_color_wave_secondary(brush);
    }
    if let Some(ResolvedToken::Dimension(px)) = theme.tokens.get("dimension.icon") {
        g.set_dimension_icon(*px);
    }
    if let Some(ResolvedToken::Dimension(px)) = theme.tokens.get("dimension.radius") {
        g.set_dimension_radius(*px);
    }
    if let Some(ResolvedToken::Dimension(px)) = theme.tokens.get("dimension.row.l") {
        g.set_dimension_row_l(*px);
    }
    if let Some(ResolvedToken::Dimension(px)) = theme.tokens.get("dimension.row.m") {
        g.set_dimension_row_m(*px);
    }
    if let Some(ResolvedToken::Dimension(px)) = theme.tokens.get("dimension.row.s") {
        g.set_dimension_row_s(*px);
    }
    if let Some(ResolvedToken::Dimension(px)) = theme.tokens.get("dimension.row.x") {
        g.set_dimension_row_x(*px);
    }
    if let Some(ResolvedToken::Dimension(px)) = theme.tokens.get("dimension.spacing.1") {
        g.set_dimension_spacing_1(*px);
    }
    if let Some(ResolvedToken::Dimension(px)) = theme.tokens.get("dimension.spacing.2") {
        g.set_dimension_spacing_2(*px);
    }
    if let Some(ResolvedToken::Dimension(px)) = theme.tokens.get("dimension.spacing.3") {
        g.set_dimension_spacing_3(*px);
    }
    if let Some(ResolvedToken::Dimension(px)) = theme.tokens.get("dimension.spacing.4") {
        g.set_dimension_spacing_4(*px);
    }
    if let Some(ResolvedToken::Dimension(px)) = theme.tokens.get("dimension.spacing.5") {
        g.set_dimension_spacing_5(*px);
    }
}