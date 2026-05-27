import { invoke } from "@tauri-apps/api/core";

export type ImageDimensions = {
  width: number;
  height: number;
};

export type ImagePreviewResult = ImageDimensions & {
  path: string;
};

export function formatImageSize(dims: ImageDimensions | undefined): string {
  if (dims == null) return "…";
  return `${dims.width}×${dims.height}`;
}

export async function fetchImagePreview(
  path: string,
  maxPx: number,
): Promise<ImagePreviewResult> {
  return invoke<ImagePreviewResult>("get_image_preview", {
    request: { path, maxPx },
  });
}
