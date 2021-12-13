
#include <iostream>
#include <immintrin.h>

static __m128i m256_haddx4(
    __m256i sum0, __m256i sum1, __m256i sum2, __m256i sum3,
    __m128i bias) {

  sum0 = _mm256_hadd_epi32(sum0, sum1);
  sum2 = _mm256_hadd_epi32(sum2, sum3);

  sum0 = _mm256_hadd_epi32(sum0, sum2);

  __m128i sum128lo = _mm256_castsi256_si128(sum0);
  __m128i sum128hi = _mm256_extracti128_si256(sum0, 1);

  return _mm_add_epi32(_mm_add_epi32(sum128lo, sum128hi), bias);
}

int main() {
    using weight_vec_t = __m256i;

    //alignas(64) std::int32_t biases[8];
    //alignas(64) std::int8_t weights[8 * 2048];

    //const weight_vec_t* weightvec = reinterpret_cast<const weight_vec_t*>(weights);

    //std::cout << "x = " << x << std::endl;

    return 0;
}
