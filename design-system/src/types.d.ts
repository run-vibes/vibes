/// <reference types="vitest/globals" />
/// <reference types="@testing-library/jest-dom" />

// CSS Modules
declare module '*.module.css' {
  const classes: { readonly [key: string]: string };
  export default classes;
}
