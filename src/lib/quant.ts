export type QuantInfo = {
  code: string;
  quality: 1 | 2 | 3 | 4 | 5;
  hint: string;
};

const TABLE: { match: RegExp; code: string; quality: 1 | 2 | 3 | 4 | 5; hint: string }[] = [
  { match: /\b(f32|fp32)\b/i, code: "F32", quality: 5, hint: "full precision, largest" },
  { match: /\b(f16|fp16|bf16)\b/i, code: "F16", quality: 5, hint: "full precision" },
  { match: /\bq8_0\b/i, code: "Q8_0", quality: 5, hint: "near lossless" },
  { match: /\bq6_k\b/i, code: "Q6_K", quality: 4, hint: "very high quality" },
  { match: /\bq5_k_m\b/i, code: "Q5_K_M", quality: 4, hint: "high quality" },
  { match: /\bq5_k_s\b/i, code: "Q5_K_S", quality: 4, hint: "high quality, smaller" },
  { match: /\bq5_0\b/i, code: "Q5_0", quality: 4, hint: "high quality" },
  { match: /\bq5_1\b/i, code: "Q5_1", quality: 4, hint: "high quality" },
  { match: /\bq4_k_m\b/i, code: "Q4_K_M", quality: 3, hint: "balanced — common pick" },
  { match: /\bq4_k_s\b/i, code: "Q4_K_S", quality: 3, hint: "balanced, smaller" },
  { match: /\bq4_0\b/i, code: "Q4_0", quality: 3, hint: "balanced" },
  { match: /\bq4_1\b/i, code: "Q4_1", quality: 3, hint: "balanced" },
  { match: /\bq3_k_l\b/i, code: "Q3_K_L", quality: 2, hint: "compressed" },
  { match: /\bq3_k_m\b/i, code: "Q3_K_M", quality: 2, hint: "compressed" },
  { match: /\bq3_k_s\b/i, code: "Q3_K_S", quality: 2, hint: "compressed, small" },
  { match: /\bq2_k\b/i, code: "Q2_K", quality: 1, hint: "heavy compression" },
  { match: /\biq4_xs\b/i, code: "IQ4_XS", quality: 3, hint: "smart compression" },
  { match: /\biq3_m\b/i, code: "IQ3_M", quality: 2, hint: "smart compression" },
  { match: /\biq3_xxs\b/i, code: "IQ3_XXS", quality: 2, hint: "tight smart compression" },
  { match: /\biq2_m\b/i, code: "IQ2_M", quality: 1, hint: "extreme compression" },
  { match: /\biq2_xs\b/i, code: "IQ2_XS", quality: 1, hint: "extreme compression" },
  { match: /\biq2_xxs\b/i, code: "IQ2_XXS", quality: 1, hint: "extreme compression" },
  { match: /\biq1_m\b/i, code: "IQ1_M", quality: 1, hint: "experimental compression" },
  { match: /\biq1_s\b/i, code: "IQ1_S", quality: 1, hint: "experimental compression" },
];

export function parseQuant(filename: string): QuantInfo | null {
  for (const entry of TABLE) {
    if (entry.match.test(filename)) {
      return { code: entry.code, quality: entry.quality, hint: entry.hint };
    }
  }
  return null;
}

export function quantTier(q: QuantInfo | null): "high" | "mid" | "low" {
  if (!q) return "mid";
  if (q.quality >= 4) return "high";
  if (q.quality >= 3) return "mid";
  return "low";
}
