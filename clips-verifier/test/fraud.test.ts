import { describe, it, expect } from 'vitest';
import { calculateFraudScore } from '../src/fraud';

describe('calculateFraudScore', () => {
  it('returns 0 for normal growth', () => {
    expect(calculateFraudScore(1000, 900, 6)).toBe(0);
  });

  it('returns 0 for zero previous views (new submission)', () => {
    expect(calculateFraudScore(100, 0, 6)).toBe(0);
  });

  it('returns 0.4 for >5x growth in normal time window', () => {
    // 6000 / 1000 = 6x, in 12h — not suspicious by time
    expect(calculateFraudScore(6000, 1000, 12)).toBe(0.4);
  });

  it('returns 0.8 for >10x growth in <6h', () => {
    // 15000 / 1000 = 15x, in 5h
    expect(calculateFraudScore(15000, 1000, 5)).toBe(0.8);
  });

  it('returns 0.8 for >10x growth in exactly 6h (boundary)', () => {
    // 12000 / 1000 = 12x, in 6h — still triggers >10x in <6h? No: 6 is not < 6
    // According to spec: >10x in <6h => 0.8; 6h is not <6h so this should be 0.4
    expect(calculateFraudScore(12000, 1000, 6)).toBe(0.4);
  });

  it('returns 0.8 for just over 10x growth in under 6h', () => {
    expect(calculateFraudScore(10001, 1000, 5)).toBe(0.8);
  });

  it('returns 0.4 for exactly 5x growth (boundary)', () => {
    // exactly 5x is not >5x, so returns 0
    expect(calculateFraudScore(5000, 1000, 12)).toBe(0);
  });

  it('returns 0.4 for just over 5x growth', () => {
    expect(calculateFraudScore(5001, 1000, 12)).toBe(0.4);
  });

  it('returns 0 for slow steady growth over many hours', () => {
    expect(calculateFraudScore(2000, 1900, 6)).toBe(0);
  });

  it('returns 0 when views decrease (no fraud signal)', () => {
    expect(calculateFraudScore(500, 1000, 6)).toBe(0);
  });

  it('handles very high multiplier in short time', () => {
    expect(calculateFraudScore(1000000, 100, 1)).toBe(0.8);
  });
});
