//! MCP prompts — single declarative table. Each prompt has a fixed text body
//! ported verbatim from the Go source.

pub struct PromptDef {
    pub name: &'static str,
    pub description: &'static str,
    pub body: &'static str,
}

pub fn all() -> &'static [PromptDef] {
    PROMPTS
}

pub fn find(name: &str) -> Option<&'static PromptDef> {
    PROMPTS.iter().find(|p| p.name == name)
}

static PROMPTS: &[PromptDef] = &[
    PromptDef {
        name: "read_design_strategy",
        description: "Best practices for reading Figma designs with figma-mcp-rust",
        body: include_str!("../prompts/read_design_strategy.md"),
    },
    PromptDef {
        name: "design_strategy",
        description: "Best practices for working with Figma designs",
        body: include_str!("../prompts/design_strategy.md"),
    },
    PromptDef {
        name: "text_replacement_strategy",
        description: "Systematic approach for replacing text in Figma designs",
        body: include_str!("../prompts/text_replacement_strategy.md"),
    },
    PromptDef {
        name: "annotation_conversion_strategy",
        description: "Strategy for converting manual annotations to Figma's native annotations",
        body: include_str!("../prompts/annotation_conversion_strategy.md"),
    },
    PromptDef {
        name: "swap_overrides_instances",
        description: "Strategy for transferring overrides between component instances in Figma",
        body: include_str!("../prompts/swap_overrides_instances.md"),
    },
    PromptDef {
        name: "reaction_to_connector_strategy",
        description: "Strategy for analyzing Figma prototype reactions and mapping interaction flows",
        body: include_str!("../prompts/reaction_to_connector_strategy.md"),
    },
    PromptDef {
        name: "style_audit_strategy",
        description: "Audit a design for nodes using raw values instead of linked styles or variables",
        body: include_str!("../prompts/style_audit_strategy.md"),
    },
    PromptDef {
        name: "bulk_rename_strategy",
        description: "Rename nodes across a design following a naming convention",
        body: include_str!("../prompts/bulk_rename_strategy.md"),
    },
    PromptDef {
        name: "design_token_generation_strategy",
        description: "Extract raw values from an existing design and build a structured variable + style token system",
        body: include_str!("../prompts/design_token_generation_strategy.md"),
    },
    PromptDef {
        name: "generate_color_palette",
        description: "Generate a complete semantic color palette (primitive scale + semantic aliases) from one or more brand colors",
        body: include_str!("../prompts/generate_color_palette.md"),
    },
    PromptDef {
        name: "generate_type_scale",
        description: "Generate a complete typography scale (text styles) from a base font and size",
        body: include_str!("../prompts/generate_type_scale.md"),
    },
    PromptDef {
        name: "generate_component_variants",
        description: "Generate design variants of an existing component or frame (size, color, state, theme)",
        body: include_str!("../prompts/generate_component_variants.md"),
    },
];
