# rt

This example is a simple pathtracer supporting planes and spheres with Lambertian materials
illuminated by point light sources. It shows more complicated interoperability with Rust code
where we have corresponding Rust and ISPC side structs working together in the program.

Note that here for dynamic dispatch I'm using enums to tag the types of the geometry, lights
and materials since the base structs are shared with Rust and it gets a bit ugly w/ ISPC's exported
function pointers. Another option is to have the `ispc_equiv` pointers be just void pointers
so the type and function pointers and such don't need to be bound in Rust and are more opaque.

Some examples scenes can be found under [`scenes/`](scenes/) which you can use as a guide if you
want to create your own. The images for the scenes shown are at higher resolution and samples
per pixel than the provided ones are but that can be changed by adjusting the width, height and
n\_samples elements of the scene.

Sphere on Plane:
![sphere on plane](http://i.imgur.com/zzS4pby.png)

R Spheres:
![r spheres](http://i.imgur.com/KKANc93.png)

