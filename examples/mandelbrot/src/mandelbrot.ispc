// Returns the count at which we exited the loop testing if the number is in the set
inline int mandel(float c_real, float c_imag, int max_iters){
	int i = 0;
	float real = c_real;
	float imag = c_imag;
	for (; i < max_iters; ++i){
		if (real * real + imag * imag > 4.f){
			break;
		}
		// We only actual care about the execution mask when incrementing i and
		// doing the break test so this will save a bit of masking
		unmasked {
			float next_r = real * real - imag * imag + c_real;
			float next_i = 2.f * real * imag + c_imag;
			real = next_r;
			imag = next_i;
		}
	}
	return i;
}
// Compute the Mandelbrot set on the real axis from [x0, x1] and imaginary
// axis from [y0, y1] to produce a width x height image. The number of iterations
// spent before finding a point is not in the set is written to counts
export void mandelbrot(const uniform float x0, const uniform float x1,
		const uniform float y0, const uniform float y1,
		const uniform int width, const uniform int height,
		const uniform int max_iters, uniform int counts[])
{
	// Width and height of a pixel in the space
	float dx = (x1 - x0) / width;
	float dy = (y1 - y0) / height;
	for (uniform int j = 0; j < height; ++j){
		// Loop through each scanline in parallel
		foreach (i = 0 ... width){
			float x = x0 + i * dx;
			float y = y0 + j * dy;
			counts[j * width + i] = mandel(x, y, max_iters);
		}
	}
}

