// Write helpers — utilities used exclusively by write handlers.

export const hexToRgb = (hex: string) => {
  const clean = hex.replace("#", "");
  return {
    r: parseInt(clean.slice(0, 2), 16) / 255,
    g: parseInt(clean.slice(2, 4), 16) / 255,
    b: parseInt(clean.slice(4, 6), 16) / 255,
    a: clean.length >= 8 ? parseInt(clean.slice(6, 8), 16) / 255 : 1,
  };
};

export const makeSolidPaint = (colorInput: any, opacityOverride?: number): SolidPaint => {
  const { r, g, b, a } = typeof colorInput === "string"
    ? hexToRgb(colorInput)
    : { r: colorInput.r, g: colorInput.g, b: colorInput.b, a: colorInput.a != null ? colorInput.a : 1 };
  const eff = opacityOverride != null ? opacityOverride : a;
  const paint: any = { type: "SOLID", color: { r, g, b } };
  if (eff !== 1) paint.opacity = eff;
  return paint;
};

export const getParentNode = async (parentId: string | undefined) => {
  if (!parentId) return figma.currentPage;
  const parent = await figma.getNodeByIdAsync(parentId);
  if (!parent) throw new Error(`Parent node not found: ${parentId}`);
  if (!("appendChild" in parent)) throw new Error(`Node ${parentId} cannot have children`);
  return parent as ChildrenMixin & BaseNode;
};

export const applyAutoLayout = (frame: FrameNode, p: any) => {
  if (p.layoutMode != null) frame.layoutMode = p.layoutMode;
  if (p.paddingTop != null) frame.paddingTop = Number(p.paddingTop);
  if (p.paddingRight != null) frame.paddingRight = Number(p.paddingRight);
  if (p.paddingBottom != null) frame.paddingBottom = Number(p.paddingBottom);
  if (p.paddingLeft != null) frame.paddingLeft = Number(p.paddingLeft);
  if (p.itemSpacing != null) frame.itemSpacing = Number(p.itemSpacing);
  if (frame.layoutMode !== "NONE") {
    if (p.primaryAxisAlignItems) frame.primaryAxisAlignItems = p.primaryAxisAlignItems;
    if (p.counterAxisAlignItems) frame.counterAxisAlignItems = p.counterAxisAlignItems;
    if (p.primaryAxisSizingMode) frame.primaryAxisSizingMode = p.primaryAxisSizingMode;
    if (p.counterAxisSizingMode) frame.counterAxisSizingMode = p.counterAxisSizingMode;
    if (p.layoutWrap) frame.layoutWrap = p.layoutWrap;
    if (p.counterAxisSpacing != null && frame.layoutWrap === "WRAP") {
      frame.counterAxisSpacing = Number(p.counterAxisSpacing);
    }
  }
};

export const base64ToBytes = (b64: string) => {
  const chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
  const lookup: Record<string, number> = {};
  for (let i = 0; i < chars.length; i++) lookup[chars[i]] = i;
  const padded = b64.replace(/[^A-Za-z0-9+/=]/g, "");
  const clean = padded.replace(/=/g, "");
  let outLen = Math.floor(padded.length * 3 / 4);
  if (padded.endsWith("==")) outLen -= 2;
  else if (padded.endsWith("=")) outLen -= 1;
  const bytes = new Uint8Array(outLen);
  let j = 0;
  for (let i = 0; i < clean.length; i += 4) {
    const a = lookup[clean[i]] || 0;
    const bv = lookup[clean[i + 1]] || 0;
    const c = lookup[clean[i + 2]] || 0;
    const d = lookup[clean[i + 3]] || 0;
    bytes[j++] = (a << 2) | (bv >> 4);
    if (j < outLen) bytes[j++] = ((bv & 15) << 4) | (c >> 2);
    if (j < outLen) bytes[j++] = ((c & 3) << 6) | d;
  }
  return bytes;
};
