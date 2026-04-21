---
name: Obsidian Neon
colors:
  surface: '#131313'
  surface-dim: '#131313'
  surface-bright: '#393939'
  surface-container-lowest: '#0e0e0e'
  surface-container-low: '#1c1b1b'
  surface-container: '#201f1f'
  surface-container-high: '#2a2a2a'
  surface-container-highest: '#353534'
  on-surface: '#e5e2e1'
  on-surface-variant: '#cfc2d6'
  inverse-surface: '#e5e2e1'
  inverse-on-surface: '#313030'
  outline: '#988d9f'
  outline-variant: '#4d4354'
  surface-tint: '#ddb7ff'
  primary: '#ddb7ff'
  on-primary: '#490080'
  primary-container: '#b76dff'
  on-primary-container: '#400071'
  inverse-primary: '#842bd2'
  secondary: '#b9f231'
  on-secondary: '#253500'
  secondary-container: '#9ed500'
  on-secondary-container: '#405800'
  tertiary: '#00daf3'
  on-tertiary: '#00363d'
  tertiary-container: '#009fb2'
  on-tertiary-container: '#002f35'
  error: '#ffb4ab'
  on-error: '#690005'
  error-container: '#93000a'
  on-error-container: '#ffdad6'
  primary-fixed: '#f0dbff'
  primary-fixed-dim: '#ddb7ff'
  on-primary-fixed: '#2c0051'
  on-primary-fixed-variant: '#6900b3'
  secondary-fixed: '#bbf534'
  secondary-fixed-dim: '#a1d806'
  on-secondary-fixed: '#141f00'
  on-secondary-fixed-variant: '#384e00'
  tertiary-fixed: '#9cf0ff'
  tertiary-fixed-dim: '#00daf3'
  on-tertiary-fixed: '#001f24'
  on-tertiary-fixed-variant: '#004f58'
  background: '#131313'
  on-background: '#e5e2e1'
  surface-variant: '#353534'
typography:
  headline-xl:
    fontFamily: Inter
    fontSize: 40px
    fontWeight: '800'
    lineHeight: 48px
    letterSpacing: -0.02em
  headline-lg:
    fontFamily: Inter
    fontSize: 24px
    fontWeight: '700'
    lineHeight: 32px
    letterSpacing: -0.01em
  body-md:
    fontFamily: Inter
    fontSize: 16px
    fontWeight: '400'
    lineHeight: 24px
  body-sm:
    fontFamily: Inter
    fontSize: 14px
    fontWeight: '400'
    lineHeight: 20px
  label-bold:
    fontFamily: Inter
    fontSize: 12px
    fontWeight: '700'
    lineHeight: 16px
    letterSpacing: 0.05em
  mono-label:
    fontFamily: Inter
    fontSize: 11px
    fontWeight: '500'
    lineHeight: 12px
    letterSpacing: 0.1em
rounded:
  sm: 0.125rem
  DEFAULT: 0.25rem
  md: 0.375rem
  lg: 0.5rem
  xl: 0.75rem
  full: 9999px
spacing:
  unit: 4px
  gutter: 16px
  margin: 24px
  panel-padding: 20px
  stack-gap: 12px
---

## Brand & Style

The design system is engineered for a high-performance gaming audience, focusing on speed, precision, and immersion. The brand personality is "Stealth-Tech"—sophisticated and unobtrusive during gameplay, yet vibrant and energetic when interacting with captured content. 

The aesthetic leverages **Glassmorphism** and **High-Contrast** elements. By combining deep, void-like blacks with ultra-vibrant accents, the UI mirrors the hardware aesthetics of modern gaming rigs. Visual interest is generated through depth—using semi-transparent layers and backdrop filters to maintain context of the underlying "game state" while navigating the app.

## Colors

The palette is strictly dark-mode, anchored by a deep charcoal base to minimize eye strain during long gaming sessions.

- **Primary (Electric Purple):** Used for primary actions, progress bars, and recording indicators.
- **Secondary (Acid Green):** Derived from the reference style, used for "Live" status, success states, and highlight callouts to provide a sharp, aggressive contrast.
- **Accent (Cyan):** Reserved for secondary interactive elements or specific "Sync" states.
- **Neutral/Surface:** A range of deep grays and blacks. Surfaces utilize #121212 with varying opacities to create the glass effect.
- **Text:** Pure white (#FFFFFF) for maximum legibility against dark backgrounds, with reduced opacity for secondary information.

## Typography

This design system utilizes **Inter** exclusively to maintain a utilitarian and systematic feel. The hierarchy is established through extreme weight variance rather than typeface changes.

- **Headlines:** Use Extra Bold (800) weights with tighter letter-spacing to create a "dense" and powerful look.
- **Labels:** Small labels and metadata (like timestamps or FPS counters) should use uppercase with increased letter-spacing to mimic digital HUDs.
- **Anti-Aliasing:** Ensure font-smoothing is enabled to maintain the "Sleek" requirement on high-resolution displays.

## Layout & Spacing

The layout follows a **Fluid Grid** model with strict 4px increments. This allows the app to scale from a small overlay to a full-screen library view.

- **Layout Structure:** A persistent sidebar (240px) for navigation, with a flexible content area.
- **Gaps:** Use a standard 16px gutter between cards in a grid.
- **Density:** High density is preferred. Elements are packed closely together to maximize the amount of content (clips) visible on the screen at once.

## Elevation & Depth

Depth is conveyed through **Glassmorphism** and **Tonal Layering** rather than traditional shadows.

1. **The Void (Base):** #000000. Used for the global background.
2. **The Surface (Tier 1):** #121212 at 80% opacity with a 20px backdrop-blur. Used for main panels.
3. **The Card (Tier 2):** #FFFFFF at 5% opacity with a 1px solid border at 10% opacity. This creates a "glass" edge.
4. **Interaction (Floating):** When an element is hovered, it should increase in opacity and gain a subtle outer glow (0px 0px 15px) tinted with the primary purple color.

## Shapes

The shape language is **Soft** but controlled. A 0.25rem (4px) base radius is used for buttons and small inputs to maintain a technical, precise feel. 

- **Cards:** Use `rounded-lg` (8px) to soften the large containers.
- **Status Indicators:** Use 100% rounding (pills) for "Live" tags and notification badges.
- **Borders:** Every glass container must have a 1px "inner-light" border to define its shape against the dark background.

## Components

### Buttons
- **Primary:** Solid Vibrant Purple (#A855F7) with white text. High-gloss finish.
- **Secondary:** Ghost style. Transparent background with a 1px white border at 20% opacity. 

### Cards (Clip Thumbnails)
- Aspect ratio 16:9.
- Use a 1px inner border to simulate glass.
- Hover state: Scale transform (1.02x) and an increase in backdrop-blur intensity.

### Input Fields
- Dark, recessed appearance. #000000 background with a 1px border that glows purple on focus.
- Micro-copy (labels) should be in the `mono-label` style.

### Progress Bars & Sliders
- Use the Secondary Acid Green (#BEF837) for the "active" track to signify motion and life.

### Chips & Tags
- Small, uppercase, bold text.
- Backgrounds should be semi-transparent versions of the category color (e.g., 10% opacity purple for a "Highlight" tag).

### Key Component: The "Sync" Indicator
- A distinctive circular element using a rotating gradient of Purple to Cyan to show active background uploading.