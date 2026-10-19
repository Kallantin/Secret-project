import { describe, expect, it } from 'vitest';
import { packImages } from './lib/pack.js';

describe('packImages', () => {
  it('packs fitting images into a single page', () => {
    const images = [
      { key: 'a', file: 'a.png', sourcePath: '/a.png', width: 100, height: 100 },
      { key: 'b', file: 'b.png', sourcePath: '/b.png', width: 100, height: 100 },
    ];
    const { pages, skipped } = packImages(images, 256, 2);
    expect(pages).toHaveLength(1);
    expect(pages[0].regions).toHaveLength(2);
    expect(skipped).toHaveLength(0);
    expect(pages[0].regions[0]).toMatchObject({ key: 'a', x: 0, y: 0 });
    expect(pages[0].regions[1]).toMatchObject({ key: 'b', x: 102, y: 0 });
  });

  it('skips images exceeding atlas size', () => {
    const images = [
      { key: 'big', file: 'big.png', sourcePath: '/big.png', width: 4096, height: 4096 },
      { key: 'small', file: 'small.png', sourcePath: '/small.png', width: 100, height: 100 },
    ];
    const { pages, skipped } = packImages(images, 2048, 2);
    expect(skipped).toHaveLength(1);
    expect(skipped[0].key).toBe('big');
    expect(skipped[0].reason).toBe('image exceeds atlas size 2048');
    expect(pages).toHaveLength(1);
    expect(pages[0].regions).toHaveLength(1);
    expect(pages[0].regions[0].key).toBe('small');
  });

  it('spills into multiple pages', () => {
    const images = Array.from({ length: 10 }, (_, i) => ({
      key: `img${i}`,
      file: `img${i}.png`,
      sourcePath: `/img${i}.png`,
      width: 1200,
      height: 1200,
    }));
    const { pages, skipped } = packImages(images, 2048, 0);
    expect(skipped).toHaveLength(0);
    expect(pages.length).toBeGreaterThan(1);
    const packed = pages.reduce((total, page) => total + page.regions.length, 0);
    expect(packed).toBe(10);
  });

  it('respects padding between images', () => {
    const images = [
      { key: 'a', file: 'a.png', sourcePath: '/a.png', width: 100, height: 100 },
      { key: 'b', file: 'b.png', sourcePath: '/b.png', width: 100, height: 100 },
    ];
    const { pages } = packImages(images, 256, 4);
    expect(pages[0].regions[1].x).toBe(104);
  });

  it('handles empty input', () => {
    const { pages, skipped } = packImages([], 2048, 2);
    expect(pages).toHaveLength(0);
    expect(skipped).toHaveLength(0);
  });
});
