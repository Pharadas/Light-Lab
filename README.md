This project is still in development, and as such many things are broken.

The project aims to allow users to play around with different experiments and light configurations, so that everyone can better understand light polarization and interference.

This is a highly modified fork of [jt's voxel ray caster](https://www.shadertoy.com/view/7dK3D3).

Hosted [here](https://pharadas.github.io/Light-Lab/)

This code simulates light interference between any number of [gaussian beams](https://en.wikipedia.org/wiki/Gaussian_beam), their electric field is defined by the equation: $${\mathbf {E} (r,z)}=E_{0}{\hat {\mathbf {x} }}{\frac {w_{0}}{w(z)}}\exp \left({\frac {-r^{2}}{w(z)^{2}}}\right)\exp \left(-i\left(kz+k{\frac {r^{2}}{2R(z)}}-\psi (z)\right)\right)$$
And the final intensity by the following equation: $$I(r,z)={|E(r,z)|^{2} \over 2\eta }$$

It is designed to be able to any numeber of 'optical objects' such as polarizers or retarders, since it treats the phase of the wave as a complex value.

https://github.com/Pharadas/RayCastingLightSimulation/assets/60682906/4b609ae8-0ece-4158-bb85-1a92796b9b99

Phase polarizers, retarders and rotators and modelled as jones matrices, the following video is a demonstration of two gaussian beams with the following polarizations:
```math
\mathbf {E_A}=\begin{bmatrix}1\\0\end{bmatrix}, \mathbf {E_B}=\begin{bmatrix}-1\\0\end{bmatrix}
```

However, one passes through a phase rotator of PI degrees, described with the following jones matrix:
```math
\mathbf {R}=\begin{bmatrix}cos(\pi) & -sin(\pi)\\sin(\pi) & cos(\pi)\end{bmatrix}
```

https://github.com/Pharadas/RayCastingLightSimulation/assets/60682906/9ba8dcb3-33a2-43d4-9464-6f1f53babb28

As expected, we only see constructive interference on the rays that passed through the rotator, everywhere else we see destructive interference.

Some examples:
Multiple unorganized light sources with the same polarization
![image](https://github.com/user-attachments/assets/b6603ea4-fe58-4f82-9dfd-26fc08e2838a)

Multiple organized light sources with the same polarization
![image](https://github.com/user-attachments/assets/f561c0d3-2fd2-4333-8c9a-7823211275be)

Multiple light sources in a diamond shape
![image](https://github.com/user-attachments/assets/9cbd3ed7-ffaf-455a-9134-c3f1ccdbe4d8)

3 slit experiment
![image](https://github.com/user-attachments/assets/bb666354-43a9-4514-8916-1b5da724a9d3)
Expected intensity distribution:
![image](https://github.com/user-attachments/assets/2e189db8-f561-4f0c-af2e-54720e7557c6)
