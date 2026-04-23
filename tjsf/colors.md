# Palette de couleurs - "Bordeaux Executive"

## Philosophie
Palette élégante et professionnelle avec fond blanc pur pour la clarté, sidebars anthracite pour le contraste, et accents bordeaux/or pour l'identité premium.

## Couleurs principales

### Primaire - Rouge Bordeaux
- **#800020** - `bordeaux`
  - Utilisation: Boutons principaux, serveurs sélectionnés, messages envoyés, logo
  - Hover: #660019 - `bordeaux-hover`

### Fond - Blanc Pur
- **#FFFFFF** - `bg-white`
  - Utilisation: Fond principal de l'application, zone de messages, modals
  - Clarté maximale pour le contenu

### Accent 1 - Or Vieilli
- **#C5A059** - `gold`
  - Utilisation: Badges Owner/Admin, statut en ligne, éléments VIP
  - Hover/Dark: #a88847 - `gold-dark`

### Accent 2 - Gris Anthracite
- **#2F3136** - `anthracite`
  - Utilisation: Sidebars (serveurs, channels, membres)
  - Hover/Light: #40444b - `anthracite-light`
  - Repos visuel par rapport au blanc

### Action/Info - Bleu Acier
- **#4682B4** - `steel-blue`
  - Utilisation: Liens, focus des inputs, informations non-critiques
  - Hover/Dark: #3a6a94 - `steel-blue-dark`

## Hiérarchie des couleurs

### Arrière-plans
- `bg-white` - Zone principale de contenu
- `bg-anthracite` - Sidebars et navigation
- `bg-anthracite-light` - Hover des éléments de sidebar
- `bg-gray-50` - Cartes et formulaires
- `bg-gray-100` - Inputs hover

### Texte
- `text-gray-900` - Texte principal sur fond blanc
- `text-white` - Texte sur fond anthracite
- `text-gray-700` - Labels et titres
- `text-gray-600` - Texte secondaire
- `text-gray-500` - Timestamps et métadonnées
- `text-gray-400` - Placeholder

### Bordures
- `border-gray-200` - Bordures sur fond blanc
- `border-anthracite-light` - Bordures sur fond anthracite
- `border-gray-300` - Bordures d'inputs

### Actions et états
- `bg-bordeaux` + `hover:bg-bordeaux-hover` - Boutons primaires
- `text-steel-blue` + `hover:underline` - Liens
- `focus:ring-steel-blue` - Focus des inputs
- `bg-gold` - Badges VIP et statut online
- `text-red-500` - Actions de suppression

## Exemples d'application

### Bouton primaire
```tsx
className="bg-bordeaux hover:bg-bordeaux-hover text-white"
```

### Lien
```tsx
className="text-steel-blue hover:underline"
```

### Input
```tsx
className="bg-white border border-gray-300 focus:ring-2 focus:ring-steel-blue"
```

### Badge Owner/Admin
```tsx
className="bg-gold text-gold-dark"
```

### Sidebar
```tsx
className="bg-anthracite text-white"
```