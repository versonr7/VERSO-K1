use ndarray::{Array1, Array2};
use std::f32;

pub struct CodeTransformer {
    vocab_size: usize,
    d_model: usize,
    embedding: Array2<f32>,
    w_q: Array2<f32>,
    w_k: Array2<f32>,
    w_v: Array2<f32>,
    w_o: Array2<f32>,
    w_ff: Array2<f32>,
}

impl CodeTransformer {
    pub fn new(vocab_size: usize, d_model: usize, _n_heads: usize) -> Self {
        let mut emb = Array2::zeros((vocab_size, d_model));
        for i in 0..vocab_size {
            for j in 0..d_model {
                emb[[i, j]] = ((i * 7 + j * 13) as f32 / 100.0).sin() * 0.1;
            }
        }
        let identity = Array2::from_diag(&Array1::from_vec(vec![1.0; d_model]));
        Self {
            vocab_size,
            d_model,
            embedding: emb,
            w_q: identity.clone(),
            w_k: identity.clone(),
            w_v: identity.clone(),
            w_o: identity.clone(),
            w_ff: identity,
        }
    }

    pub fn encode_tokens(&self, tokens: &[u32]) -> Array2<f32> {
        let mut out = Array2::zeros((tokens.len(), self.d_model));
        for (i, &tok) in tokens.iter().enumerate() {
            let idx = (tok as usize) % self.vocab_size;
            for j in 0..self.d_model {
                out[[i, j]] = self.embedding[[idx, j]];
            }
        }
        out
    }

    fn softmax(x: &Array2<f32>) -> Array2<f32> {
        let mut out = x.clone();
        for i in 0..x.nrows() {
            let max_val = x.row(i).iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
            let sum: f32 = x.row(i).iter().map(|&v| (v - max_val).exp()).sum();
            for j in 0..x.ncols() {
                out[[i, j]] = (x[[i, j]] - max_val).exp() / sum;
            }
        }
        out
    }

    fn matmul(a: &Array2<f32>, b: &Array2<f32>) -> Array2<f32> {
        let mut out = Array2::zeros((a.nrows(), b.ncols()));
        for i in 0..a.nrows() {
            for j in 0..b.ncols() {
                let mut sum = 0.0;
                for k in 0..a.ncols() {
                    sum += a[[i, k]] * b[[k, j]];
                }
                out[[i, j]] = sum;
            }
        }
        out
    }

    pub fn understand_code(&self, tokens: &[u32]) -> Array2<f32> {
        let x = self.encode_tokens(tokens);
        let q = Self::matmul(&x, &self.w_q);
        let k = Self::matmul(&x, &self.w_k);
        let v = Self::matmul(&x, &self.w_v);

        let kt = k.t().to_owned();
        let scores = Self::matmul(&q, &kt);
        let scale = (self.d_model as f32).sqrt();
        let scaled = scores.mapv(|v| v / scale);
        let attn = Self::softmax(&scaled);
        let attn_out = Self::matmul(&attn, &v);

        let x = x + attn_out;
        let ff_out = Self::matmul(&x, &self.w_ff);
        x + ff_out
    }

    pub fn vocab_size(&self) -> usize { self.vocab_size }
    pub fn d_model(&self) -> usize { self.d_model }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transformer_init() {
        let t = CodeTransformer::new(100, 64, 4);
        assert_eq!(t.vocab_size(), 100);
    }

    #[test]
    fn test_encode_tokens() {
        let t = CodeTransformer::new(100, 16, 2);
        let tokens = vec![1u32, 2, 3, 4];
        let emb = t.encode_tokens(&tokens);
        assert_eq!(emb.shape(), &[4, 16]);
    }

    #[test]
    fn test_understand_code() {
        let t = CodeTransformer::new(50, 32, 2);
        let tokens = vec![10u32, 20, 30];
        let out = t.understand_code(&tokens);
        assert_eq!(out.shape(), &[3, 32]);
    }
}
