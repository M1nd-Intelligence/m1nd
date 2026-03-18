export type ImageDataSource = CanvasImageSource & {
    width: number;
    height: number;
};
export declare function loadImage(source: string): Promise<HTMLImageElement>;
export declare function loadAnimation(sources: string[]): Promise<HTMLImageElement[]>;
export declare function getImageData(image: ImageDataSource): ImageData;
//# sourceMappingURL=loadImage.d.ts.map