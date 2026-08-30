import { describe, expect, it } from 'vitest';
import { basename, formatBytes, shortHash } from './format';

describe('format helpers', () => {
  it('formats byte values predictably', () => {
    expect(formatBytes(0)).toBe('0 B');
    expect(formatBytes(1024)).toBe('1.0 KB');
    expect(formatBytes(10 * 1024 * 1024)).toBe('10 MB');
  });

  it('shortens hashes without destroying small values', () => {
    expect(shortHash('abc')).toBe('abc');
    expect(shortHash('1234567890123456')).toBe('12345678…3456');
  });

  it('extracts Windows and POSIX basenames', () => {
    expect(basename('C:\\Games\\pack')).toBe('pack');
    expect(basename('/tmp/pack')).toBe('pack');
  });
});
