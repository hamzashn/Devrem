declare module "svgcanvas" {
  /** Canvas 2D API'sini taklit edip SVG üreten bağlam. */
  export class Context {
    constructor(width: number, height: number);
    getSerializedSvg(fixNamedEntities?: boolean): string;
  }
}
