/* Implementation of Improved Perlin Noise: http://mrl.nyu.edu/~perlin/noise/
 * I also found this guide helpful: http://flafla2.github.io/2014/08/09/perlinnoise.html
 */

const static uniform int PERMUTATION[512] = {
	151,160,137,91,90,15,
	131,13,201,95,96,53,194,233,7,225,140,36,103,30,69,142,8,99,37,240,21,10,23,
	190, 6,148,247,120,234,75,0,26,197,62,94,252,219,203,117,35,11,32,57,177,33,
	88,237,149,56,87,174,20,125,136,171,168, 68,175,74,165,71,134,139,48,27,166,
	77,146,158,231,83,111,229,122,60,211,133,230,220,105,92,41,55,46,245,40,244,
	102,143,54, 65,25,63,161, 1,216,80,73,209,76,132,187,208, 89,18,169,200,196,
	135,130,116,188,159,86,164,100,109,198,173,186, 3,64,52,217,226,250,124,123,
	5,202,38,147,118,126,255,82,85,212,207,206,59,227,47,16,58,17,182,189,28,42,
	223,183,170,213,119,248,152, 2,44,154,163, 70,221,153,101,155,167, 43,172,9,
	129,22,39,253, 19,98,108,110,79,113,224,232,178,185, 112,104,218,246,97,228,
	251,34,242,193,238,210,144,12,191,179,162,241, 81,51,145,235,249,14,239,107,
	49,192,214, 31,181,199,106,157,184, 84,204,176,115,121,50,45,127, 4,150,254,
	138,236,205,93,222,114,67,29,24,72,243,141,128,195,78,66,215,61,156,180,
	151,160,137,91,90,15,
	131,13,201,95,96,53,194,233,7,225,140,36,103,30,69,142,8,99,37,240,21,10,23,
	190, 6,148,247,120,234,75,0,26,197,62,94,252,219,203,117,35,11,32,57,177,33,
	88,237,149,56,87,174,20,125,136,171,168, 68,175,74,165,71,134,139,48,27,166,
	77,146,158,231,83,111,229,122,60,211,133,230,220,105,92,41,55,46,245,40,244,
	102,143,54, 65,25,63,161, 1,216,80,73,209,76,132,187,208, 89,18,169,200,196,
	135,130,116,188,159,86,164,100,109,198,173,186, 3,64,52,217,226,250,124,123,
	5,202,38,147,118,126,255,82,85,212,207,206,59,227,47,16,58,17,182,189,28,42,
	223,183,170,213,119,248,152, 2,44,154,163, 70,221,153,101,155,167, 43,172,9,
	129,22,39,253, 19,98,108,110,79,113,224,232,178,185, 112,104,218,246,97,228,
	251,34,242,193,238,210,144,12,191,179,162,241, 81,51,145,235,249,14,239,107,
	49,192,214, 31,181,199,106,157,184, 84,204,176,115,121,50,45,127, 4,150,254,
	138,236,205,93,222,114,67,29,24,72,243,141,128,195,78,66,215,61,156,180
};

float fade(const float t){
	return pow(t, 3.f) * (t * (t * 6.f - 15.f) + 10.f);
}
float lerp(const float t, const float a, const float b){
	return (1.f - t) * a + t * b;
}
float grad(const int hash, const float x, const float y, const float z){
	const int h = hash & 15;
	float u = h < 8 ? x : y;
	float v = h < 4 ? y : h == 12 || h == 14 ? x : z;
	return ((h & 1) == 0 ? u : -u) + ((h & 2) == 0 ? v : -v);
}
float noise(const float<3> p){
	// Find unit cube containing the point
	int<3> cell = {floor(p.x), floor(p.y), floor(p.z)};
	cell &= 255;

	// Relative position of this point in the cell
	const float<3> d = p - cell;
	const float<3> weight = {fade(d.x), fade(d.y), fade(d.z)};

	// Hash coordinates of the cube corners
	const int a = PERMUTATION[cell.x] + cell.y;
	const int aa = PERMUTATION[a] + cell.z;
	const int ab = PERMUTATION[a + 1] + cell.z;
	const int b = PERMUTATION[cell.x + 1] + cell.y;
	const int ba = PERMUTATION[b] + cell.z;
	const int bb = PERMUTATION[b + 1] + cell.z;

	// Compute and blend results from the cube corners
	const float g111 = grad(PERMUTATION[aa], d.x, d.y, d.z);
	const float g011 = grad(PERMUTATION[ba], d.x - 1, d.y, d.z);
	const float g101 = grad(PERMUTATION[ab], d.x, d.y - 1, d.z);
	const float g001 = grad(PERMUTATION[bb], d.x - 1, d.y - 1, d.z);
	const float g110 = grad(PERMUTATION[aa + 1], d.x, d.y, d.z - 1);
	const float g010 = grad(PERMUTATION[ba + 1], d.x - 1, d.y, d.z - 1);
	const float g100 = grad(PERMUTATION[ab + 1], d.x, d.y - 1, d.z - 1);
	const float g000 = grad(PERMUTATION[bb + 1], d.x - 1, d.y - 1, d.z - 1);
	return lerp(weight.z,
			lerp(weight.y, lerp(weight.x, g111, g011), lerp(weight.x, g101, g001)),
			lerp(weight.y, lerp(weight.x, g110, g010), lerp(weight.x, g100, g000)));
}
// TODO: why do I get such a strange/buggy image at 8 octaves?
float octave_noise(const float<3> p, const uniform int octaves, const uniform float persistence){
	float total = 0.f;
	float frequency = 1.f;
	float amplitude = 1.f;
	float max_val = 0.f;
	for (int i = 0; i < octaves; ++i){
		total += noise(p * frequency) * amplitude;
		max_val += amplitude;
		amplitude *= persistence;
		frequency *= 2.f;
	}
	return total / max_val;
}

export void perlin(const uniform float width, const uniform float height,
		const uniform int img_width, const uniform int img_height, uniform float img[]){
	uniform const float dx = width / img_width;
	uniform const float dy = height / img_height;
	foreach_tiled (j = 0 ... img_height, i = 0 ... img_width){
		float<3> p = {dx * i, dy * j, 0.2f};
		img[j * img_width + i] = octave_noise(p, 6, 0.5);
	}
}

