export interface LoadedImage {
  key: string;
  file: string;
  sourcePath: string;
  width: number;
  height: number;
}

export interface Region {
  key: string;
  sourcePath: string;
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface Page {
  page: number;
  width: number;
  height: number;
  regions: Region[];
}

export interface SkippedImage {
  key: string;
  file: string;
  width: number;
  height: number;
  reason: string;
}

export function packImages(
  images: LoadedImage[],
  size: number,
  padding: number,
): { pages: Page[]; skipped: SkippedImage[] } {
  const skipped: SkippedImage[] = [];
  const sortable = images.filter((image) => {
    if (image.width > size || image.height > size) {
      skipped.push({
        key: image.key,
        file: image.file,
        width: image.width,
        height: image.height,
        reason: `image exceeds atlas size ${size}`,
      });
      return false;
    }
    return true;
  });

  sortable.sort((left, right) => {
    if (right.height !== left.height) {
      return right.height - left.height;
    }
    if (right.width !== left.width) {
      return right.width - left.width;
    }
    return left.key.localeCompare(right.key);
  });

  const pages: Page[] = [];
  let page: Page | null = null;
  let cursorX = 0;
  let shelfY = 0;
  let shelfHeight = 0;
  let usedWidth = 0;
  let usedHeight = 0;
  const step = padding > 0 ? padding : 0;
  const startPage = (): Page => ({ page: pages.length + 1, width: size, height: size, regions: [] });

  for (const image of sortable) {
    if (page === null) {
      page = startPage();
    }
    let placed = false;
    while (!placed) {
      if (cursorX + image.width <= size && shelfY + image.height <= size) {
        page.regions.push({
          key: image.key,
          sourcePath: image.sourcePath,
          x: cursorX,
          y: shelfY,
          width: image.width,
          height: image.height,
        });
        cursorX += image.width + step;
        shelfHeight = Math.max(shelfHeight, image.height);
        usedWidth = Math.max(usedWidth, cursorX - step);
        usedHeight = Math.max(usedHeight, shelfY + image.height);
        placed = true;
      } else if (cursorX > 0) {
        cursorX = 0;
        shelfY += shelfHeight + step;
        shelfHeight = 0;
      } else {
        page.width = Math.max(usedWidth, 1);
        page.height = Math.max(usedHeight, 1);
        pages.push(page);
        page = startPage();
        cursorX = 0;
        shelfY = 0;
        shelfHeight = 0;
        usedWidth = 0;
        usedHeight = 0;
      }
    }
  }

  if (page !== null && page.regions.length > 0) {
    page.width = Math.max(usedWidth, 1);
    page.height = Math.max(usedHeight, 1);
    pages.push(page);
  }

  return { pages, skipped };
}
