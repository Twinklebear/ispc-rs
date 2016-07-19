#include "tfn.ih"

export void make_transfer_function(uniform ISPCHandle* uniform out,
		const uniform Vec3f* uniform colors, const uniform int n_colors,
		const uniform float* uniform opacity, const uniform int n_opacities)
{
	uniform TransferFunction* uniform tfn = uniform new uniform TransferFunction;
	typedef float<3> svec3f;
	tfn->colors = uniform new uniform svec3f[n_colors];
	tfn->n_colors = n_colors;
	for (uniform int i = 0; i < tfn->n_colors; ++i){
		tfn->colors[i].x = colors[i].x;
		tfn->colors[i].y = colors[i].y;
		tfn->colors[i].z = colors[i].z;
	}
	tfn->opacities = uniform new uniform float[n_opacities];
	tfn->n_opacities = n_opacities;
	for (uniform int i = 0; i < tfn->n_opacities; ++i){
		tfn->opacities[i] = opacity[i];
	}

	*out = tfn;
}
export void drop_transfer_function(uniform ISPCHandle tfn){
	uniform TransferFunction* uniform t = (uniform TransferFunction* uniform)tfn;
	delete[] t->colors;
	delete[] t->opacities;
	delete t;
}
float<4> sample_transfer_function(const uniform ISPCHandle tfn, const float val){
	uniform TransferFunction* uniform t = (uniform TransferFunction* uniform)tfn;
	float<3> color = {0, 0, 0};
	float opacity = 0;
	// Need to re-map the range of lo/hi to 0/1 then we can interpolate
	// based on val re-mapped into this range as well
	{
		const float color_idx = val * t->n_colors;
		const int lo = color_idx;
		const int hi = lo + 1;
		if (hi == t->n_colors){
			color = t->colors[lo];
		} else {
			const float t_val = color_idx - lo;
			color = (1.0 - t_val) * t->colors[lo] + t_val * t->colors[hi];
		}
	}
	{
		const float opacity_idx = val * t->n_opacities;
		const int lo = opacity_idx;
		const int hi = lo + 1;
		if (hi == t->n_opacities){
			opacity = t->opacities[lo];
		} else {
			const float t_val = opacity_idx - lo;
			opacity = (1.0 - t_val) * t->opacities[lo] + t_val * t->opacities[hi];
		}
	}
	const float<4> sample = {color.x, color.y, color.z, opacity};
	return sample;
}

