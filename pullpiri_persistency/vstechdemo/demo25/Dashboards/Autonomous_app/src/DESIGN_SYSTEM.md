# SDV Project Design System
## Theme Guide for Software Defined Vehicle Applications

This document outlines the complete design system used for the SDV Project Orchestrator suite. Use this guide to maintain visual consistency across all related applications in the SDV project ecosystem.

---

## Color Palette

### Background Colors
- **Primary Background**: `bg-gradient-to-br from-slate-900 via-slate-800 to-slate-900`
  - Used for: Main application background
- **Card Background**: `bg-slate-800/50`
  - Used for: All card containers, panels
- **Card Border**: `border-slate-700`
  - Used for: Card borders, separators
- **Secondary Background**: `bg-slate-900`
  - Used for: Nested containers, input backgrounds
- **Dark Overlay**: `bg-slate-900/50`
  - Used for: List items, secondary panels

### Primary Accent Colors
- **Blue (Primary Action)**: 
  - Main: `bg-blue-600`, `text-blue-400`
  - Hover: `hover:bg-blue-700`
  - Used for: Primary buttons, gauges, active states
- **Green (Success/Active)**:
  - Main: `bg-green-600`, `text-green-400`
  - Dark: `bg-green-950/30`
  - Border: `border-green-600/50`
  - Used for: Active states, success indicators, "good" status
- **Red (Critical/Danger)**:
  - Main: `bg-red-600`, `text-red-400`, `#ef4444`
  - Dark: `bg-red-950/30`
  - Border: `border-red-600/50`
  - Used for: Alerts, critical warnings, stop actions

### Secondary Accent Colors
- **Yellow (Warning)**:
  - Main: `bg-yellow-400`, `text-yellow-400`, `#eab308`
  - Dark: `bg-yellow-950/30`
  - Used for: Lane markers, warnings
- **Orange (Caution)**:
  - Main: `bg-orange-500`, `text-orange-400`, `#f59e0b`
  - Dark: `bg-orange-950/30`
  - Border: `border-orange-600/50`
  - Used for: Caution states, moderate warnings

### Text Colors
- **Primary Text**: `text-white`
  - Used for: Headings, primary content, important values
- **Secondary Text**: `text-slate-400`
  - Used for: Labels, descriptions, subtitles
- **Tertiary Text**: `text-slate-500`
  - Used for: Minor labels, scales, axes
- **Muted Text**: `text-slate-600`
  - Used for: Disabled states, background elements

---

## Typography

### Font Sizes
**IMPORTANT**: Do NOT use Tailwind font size classes (`text-xl`, `text-2xl`, etc.) unless specifically changing typography. Default HTML element styles are defined in `styles/globals.css`.

When you DO need to specify sizes:
- **Hero Text**: `text-5xl` (48px) - Large numerical displays
- **Page Title**: `text-3xl` (30px) - Main headings (h1)
- **Section Title**: `text-2xl` (24px) - Section headings (h2)
- **Subsection**: `text-xl` (20px) - Card titles, parameters
- **Body**: Default (16px) - Regular text
- **Small**: `text-sm` (14px) - Labels, descriptions
- **Tiny**: `text-xs` (12px) - Minor labels, scales

### Font Weight
Avoid using font weight classes. Let the default styles handle it.

---

## Component Patterns

### Cards
```
<Card className="p-6 bg-slate-800/50 border-slate-700">
```
- Standard padding: `p-6`
- Compact cards: `p-4`
- Always use semi-transparent background: `bg-slate-800/50`

### Status Cards (Active/Success)
```
<Card className="p-4 bg-green-950/30 border-green-600/50">
```
- Use color-specific dark background with `/30` opacity
- Use color-specific border with `/50` opacity

### Status Cards (Warning/Critical)
```
<Card className="p-4 bg-red-950/30 border-red-600/50">
```
```
<Card className="p-4 bg-orange-950/30 border-orange-600/50">
```

### Buttons
**Primary Action (Launch/Activate)**:
```
<Button className="bg-green-600 hover:bg-green-700">
```

**Destructive/Stop Action**:
```
<Button variant="destructive">
```

**Default**:
```
<Button variant="default">
```

---

## Icons & Visual Elements

### Icon Library
- **Package**: `lucide-react`
- **Size**: `w-5 h-5` (standard), `w-8 h-8` (large), `w-4 h-4` (small)

### Common Icon Colors
- **Primary Icons**: `text-blue-400`
- **Success Icons**: `text-green-400`, `text-green-500`
- **Warning Icons**: `text-yellow-400`
- **Alert Icons**: `text-red-400`
- **Inactive Icons**: `text-slate-400`

### Icon Containers
```
<div className="p-3 bg-blue-600 rounded-lg">
  <IconComponent className="w-8 h-8 text-white" />
</div>
```

---

## Layout Patterns

### Main Container
```
<div className="min-h-screen bg-gradient-to-br from-slate-900 via-slate-800 to-slate-900">
  <div className="container mx-auto p-6">
```

### Grid Layouts
**Two Column (Desktop)**:
```
<div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
```

**Four Column (Parameter Cards)**:
```
<div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
```

**Three Column (Details)**:
```
<div className="grid grid-cols-1 md:grid-cols-3 gap-6">
```

### Spacing
- Section spacing: `space-y-6`
- Card spacing: `gap-6` (large), `gap-4` (standard)
- Internal spacing: `gap-3`, `gap-2`
- Margin bottom for headers: `mb-8` (page), `mb-6` (section), `mb-4` (card)

---

## Status Indicators

### Color Coding System
- **Green**: Active, Optimal, Safe, Go
- **Yellow**: Warning, Moderate, Caution  
- **Orange**: Concerning, Approaching Limits
- **Red**: Critical, Dangerous, Stop, Error
- **Blue**: Normal Operation, Neutral Info
- **Slate/Gray**: Inactive, Disabled, Unknown

### Badge Patterns
```
<Badge className="bg-green-600">System Operational</Badge>
<Badge variant="destructive">Critical Alert</Badge>
<Badge variant="secondary">Inactive</Badge>
```

### Pulsing Indicators (Live/Active)
```
<div className="w-3 h-3 rounded-full bg-green-500 animate-pulse"></div>
```

### Status Icons with Text
```
<div className="flex items-center gap-3">
  <Activity className="w-5 h-5 text-green-500 animate-pulse" />
  <span className="text-green-400">Status Text</span>
</div>
```

---

## Data Visualization

### Circular Gauges
- Container: `w-48 h-48` (192px)
- Background circle: `stroke="#334155"` (slate-700)
- Progress circle: `stroke="#3b82f6"` (blue-500) or `stroke="#ef4444"` (red-500)
- Stroke width: `strokeWidth="16"`
- Center text: `text-5xl` for value, `text-slate-400` for unit

### Progress Bars
- Container: `h-2 bg-slate-900 rounded-full`
- Fill: `bg-blue-500` (normal), `bg-red-500` (warning)
- Transition: `transition-all duration-300`

### Radar/Sensor Displays
- SVG viewBox: `"0 0 200 200"`
- Grid lines: `stroke="#334155"`
- Center vehicle: `fill="#3b82f6"` (blue-500)
- Obstacles: Color-coded by distance (green safe, yellow caution, red critical)

---

## Interactive States

### Transitions
- Standard transition: `transition-all duration-300`
- Color transitions: `transition-colors duration-300`
- Smooth animations for real-time data

### Hover States
- Cards: Minimal or no hover effect (data display)
- Buttons: Standard hover darkening
- Interactive elements: `hover:bg-slate-700`

### Active/Selected States
- Background: `bg-slate-700`
- Border: Add colored border matching accent
- Text: Brighten to `text-white`

---

## Specialized Components

### Environmental/Weather Cards
```
<Card className="p-6 bg-slate-800/50 border-slate-700">
  <h3 className="text-white mb-4">Environmental Conditions</h3>
  <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
    <div>
      <p className="text-slate-400 text-sm mb-1">Label</p>
      <p className="text-white text-xl">Value</p>
    </div>
  </div>
</Card>
```

### Inactive/Empty States
```
<div className="text-center py-8">
  <div className="w-16 h-16 mx-auto mb-3 rounded-full bg-green-950/50 flex items-center justify-center">
    <div className="w-3 h-3 rounded-full bg-green-500"></div>
  </div>
  <p className="text-slate-400">Primary message</p>
  <p className="text-slate-500 text-sm">Secondary message</p>
</div>
```

---

## Iconography Associations

### By Feature
- **Vehicle/Driving**: `Car`, `Gauge`, `Navigation`
- **Status/Activity**: `Activity`, `Power`, `Zap`
- **Warnings**: `AlertTriangle`, `AlertCircle`
- **Detection**: `User` (pedestrian), `Bike` (cyclist), `Car` (vehicle)
- **Navigation**: `Navigation`, `Compass`, `MapPin`
- **Controls**: `Play`, `Pause`, `StopCircle`, `Power`

---

## Responsive Design

### Breakpoints (Tailwind defaults)
- Mobile: default (no prefix)
- Tablet: `md:` (768px+)
- Desktop: `lg:` (1024px+)

### Responsive Patterns
```
// Stack on mobile, side-by-side on desktop
grid grid-cols-1 lg:grid-cols-2

// 1 column mobile, 2 tablet, 4 desktop
grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4
```

---

## Animation Guidelines

### Pulse (Live Data)
```
animate-pulse
```
Use for: Active indicators, live data points, critical alerts

### Smooth Transitions
```
transition-all duration-300
```
Use for: Data updates, position changes, color changes

### Rotation (Steering)
```
style={{ transform: `rotate(${angle}deg)` }}
```
Combined with transition for smooth steering visualization

---

## Accessibility

### Contrast
- All text meets WCAG AA standards against dark backgrounds
- White (`text-white`) for primary content
- `text-slate-400` for secondary content (still readable)

### Interactive Elements
- Buttons have clear hover states
- Sufficient spacing for touch targets (min 44x44px)
- Icons always paired with text labels or aria-labels

---

## Usage Examples

### App Header Pattern
```tsx
<div className="flex items-center justify-between mb-8">
  <div className="flex items-center gap-3">
    <div className="p-3 bg-blue-600 rounded-lg">
      <IconComponent className="w-8 h-8 text-white" />
    </div>
    <div>
      <h1 className="text-white text-3xl">App Title</h1>
      <p className="text-slate-400">Subtitle</p>
    </div>
  </div>
  <Button>Action</Button>
</div>
```

### Status Banner Pattern
```tsx
<Card className="p-4 bg-green-950/30 border-green-600/50">
  <div className="flex items-center justify-between">
    <div className="flex items-center gap-3">
      <Activity className="w-5 h-5 text-green-500 animate-pulse" />
      <span className="text-green-400">System Status Message</span>
    </div>
    <Badge className="bg-green-600">Status</Badge>
  </div>
</Card>
```

### Parameter Display Pattern
```tsx
<div>
  <p className="text-slate-400 text-sm mb-1">Parameter Name</p>
  <p className="text-white text-xl">Value</p>
</div>
```

---

## Key Principles

1. **Dark Theme**: Always use dark backgrounds (slate-900, slate-800)
2. **Semi-transparency**: Use `/50` or `/30` opacity for layering
3. **Color Meaning**: Green=good, Yellow=caution, Red=danger, Blue=info
4. **Consistent Spacing**: Use increments of 4 (gap-2, gap-4, gap-6)
5. **Smooth Transitions**: Always animate state changes (300ms duration)
6. **White Primary Text**: Important data always in white
7. **Slate Secondary**: Labels and descriptions in slate-400
8. **Border Consistency**: Always use slate-700 for neutral borders
9. **Card Pattern**: Always `bg-slate-800/50 border-slate-700` for standard cards
10. **Gradient Background**: Main app always uses the slate gradient

---

## Quick Reference: Common Class Combinations

**Standard Card**: `p-6 bg-slate-800/50 border-slate-700`

**Success Card**: `p-4 bg-green-950/30 border-green-600/50`

**Warning Card**: `p-4 bg-orange-950/30 border-orange-600/50`

**Critical Card**: `p-4 bg-red-950/30 border-red-600/50`

**Icon Container**: `p-3 bg-blue-600 rounded-lg`

**Live Indicator**: `w-3 h-3 rounded-full bg-green-500 animate-pulse`

**Parameter Label**: `text-slate-400 text-sm`

**Parameter Value**: `text-white text-xl`

**Section Title**: `text-white mb-4`

**Grid 2-Col**: `grid grid-cols-1 lg:grid-cols-2 gap-6`

**Grid 4-Col**: `grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4`

---

## File Structure Pattern

```
/App.tsx                 - Main orchestrator with launch button
/components/
  MainDashboard.tsx      - Main feature dashboard
  SpecificFeature.tsx    - Feature-specific components
  ParameterCard.tsx      - Reusable parameter display
  ui/                    - Shadcn components (don't modify)
```

---

**Last Updated**: 2025-11-12
**Version**: 1.0
**Project**: SDV (Software Defined Vehicle) Project Orchestrator Suite
