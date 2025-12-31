/// <reference types="vite/client" />

// CSS Modules - needed for @vibes/design-system components
declare module '*.module.css' {
  const classes: { readonly [key: string]: string };
  export default classes;
}

// Plain CSS imports
declare module '*.css' {
  const content: string;
  export default content;
}
