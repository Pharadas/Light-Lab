precision mediump float;
precision mediump int;
in vec4 v_color;

uniform vec2 u_rotation;
uniform vec3 position; 
uniform uint objects[3000];
uniform uint buckets[1000];
// uniform vec2 viewport_dimensions;
uniform float time;

layout(location = 0) out vec4 out_color;
layout(location = 1) out vec4 object_found;

// Constants definitions =================================
const int MAX_RAY_STEPS = 1000;
const ivec3 WORLD_SIZE = ivec3(200, 200, 200);
const float PI = 3.1416;
const uint U32_MAX = uint(4294967295);

// WorldObject.type possible values
const uint CUBE = uint(0);                        // Filled cube that can only be in uvec3 positions
const uint SQUARE_WALL = uint(1);                 // Infinitesimally thin square wall
const uint ROUND_WALL = uint(2);                  // Infinitesimally thin round wall
const uint LIGHT_SOURCE = uint(3);                // Sphere that represents a light source
const uint OPTICAL_OBJECT_CUBE = uint(4);         // An object represented using a jones matrix
const uint OPTICAL_OBJECT_SQUARE_WALL = uint(5);  // An object represented using a jones matrix
const uint OPTICAL_OBJECT_ROUND_WALL = uint(6);   // An object represented using a jones matrix

struct gsl_complex {
  float dat[2];
};

struct ComplexNumber {
  vec2 dat;
};

// Complex matrix =
// |a b|
// |c d|
struct Complex2x2Matrix {
  ComplexNumber a;
  ComplexNumber b;
  ComplexNumber c;
  ComplexNumber d;
};

struct Polarization {
  ComplexNumber Ex;
  ComplexNumber Ey;
};

// Struct definitions ====================================
struct WorldObject {
  uint type;
  vec3 center;
  // top_left and bottom_right will only be relevant if the object is a
  // square of round wall
  vec3 top_left;
  vec3 bottom_right;
  // Will only be relevant if the object is a round wall
  // or a light source
  // will be the radius of the sphere if it's a light source
  // and the radius of the wall
  float radius;
  // Will only be relevant if it's a light source
  Polarization polarization;
  // Will only be relevant if it's an optical object
  Complex2x2Matrix jones_matrix;
};

uniform WorldObject objects_definitions[100];

struct RayObject {
  // current direction of the ray
  vec3 dir;
  // original position of the ray
  vec3 pos;
  // current position of the ray truncated
  ivec3 map_pos;

  vec3 delta_dist;
  ivec3 step;
  vec3 side_dist; 

  // current axis in which the ray entered the block
  bvec3 mask;

  float distance_traveled;
  vec3 current_real_position;
  vec3 original_dir;

  bool ended_in_hit;

  vec4 color;
  uint object_hit;
  // Complex2x2Matrix optical_objects_found_product;
  // int optical_objects_through_which_it_passed;
};

/**********************************************************************
 * GSL Complex numbers
 **********************************************************************/

#  define GSL_REAL(z)              ((z).dat[0])
#  define GSL_IMAG(z)              ((z).dat[1])
#  define GSL_COMPLEX_P(z)        ((z).dat)
#  define GSL_COMPLEX_P_REAL(z)   ((z).dat[0])
#  define GSL_COMPLEX_P_IMAG(z)   ((z).dat[1])
#  define GSL_COMPLEX_EQ(z1,z2)    (((z1).dat[0] == (z2).dat[0]) && ((z1).dat[1] == (z2).dat[1]))

#  define GSL_SET_COMPLEX(z,x,y)  {(z).dat[0]=(x); (z).dat[1]=(y);}
#  define GSL_SET_REAL(z,x)       {(z).dat[0]=(x);}
#  define GSL_SET_IMAG(z,y)       {(z).dat[1]=(y);}

// defined by me by hand (so could be wrong)
#  define gsl_isinf(x) (isinf(x))
#  define GSL_POSINF (1.0/0.0)
#  define GSL_MAX(a, b) max(a, b)
#  define M_PI       3.14159265358979323846264338328      /* pi */
#  define M_PI_2     1.57079632679489661923132169164      /* pi/2 */
#  define M_PI_4     0.78539816339744830961566084582     /* pi/4 */

float gsl_hypot (const float x, const float y)
{
  float xabs = abs (x) ;
  float yabs = abs (y) ;
  float min, max;

  /* Follow the optional behavior of the ISO C standard and return
     +Inf when any of the argument is +-Inf, even if the other is NaN.
     http://pubs.opengroup.org/onlinepubs/009695399/functions/hypot.html */
  if (gsl_isinf(x) || gsl_isinf(y)) 
    {
      return GSL_POSINF;
    }

  if (xabs < yabs) {
    min = xabs ;
    max = yabs ;
  } else {
    min = yabs ;
    max = xabs ;
  }

  if (min == 0.0) 
    {
      return max ;
    }

  {
    float u = min / max ;
    return max * sqrt (1.0 + u * u) ;
  }
}

float
gsl_hypot3(const float x, const float y, const float z)
{
  float xabs = abs (x);
  float yabs = abs (y);
  float zabs = abs (z);
  float w = GSL_MAX(xabs, GSL_MAX(yabs, zabs));

  if (w == 0.0)
    {
      return (0.0);
    }
  else
    {
      float r = w * sqrt((xabs / w) * (xabs / w) +
                         (yabs / w) * (yabs / w) +
                         (zabs / w) * (zabs / w));
      return r;
    }
}

gsl_complex
gsl_complex_polar (float r, float theta)
{                               /* return z = r exp(i theta) */
  gsl_complex z;
  GSL_SET_COMPLEX (z, r * cos (theta), r * sin (theta));
  return z;
}

/**********************************************************************
 * Properties of complex numbers
 **********************************************************************/

float
gsl_complex_arg (gsl_complex z)
{                               /* return arg(z),  -pi < arg(z) <= +pi */
  float x = GSL_REAL (z);
  float y = GSL_IMAG (z);

  if (x == 0.0 && y == 0.0)
    {
      return 0.0;
    }

  return atan (y, x);
}

float
gsl_complex_abs (gsl_complex z)
{                               /* return |z| */
  return gsl_hypot (GSL_REAL (z), GSL_IMAG (z));
}

float
gsl_complex_abs2 (gsl_complex z)
{                               /* return |z|^2 */
  float x = GSL_REAL (z);
  float y = GSL_IMAG (z);

  return (x * x + y * y);
}

float
gsl_complex_logabs (gsl_complex z)
{                               /* return log|z| */
  float xabs = abs  (GSL_REAL (z));
  float yabs = abs  (GSL_IMAG (z));
  float max, u;

  if (xabs >= yabs)
    {
      max = xabs;
      u = yabs / xabs;
    }
  else
    {
      max = yabs;
      u = xabs / yabs;
    }

  /* Handle underflow when u is close to 0 */

  // changed log1p to log
  return log (max) + 0.5 * log (u * u);
}

/***********************************************************************
 * Complex arithmetic operators
 ***********************************************************************/

gsl_complex
gsl_complex_add (gsl_complex a, gsl_complex b)
{                               /* z=a+b */
  float ar = GSL_REAL (a), ai = GSL_IMAG (a);
  float br = GSL_REAL (b), bi = GSL_IMAG (b);

  gsl_complex z;
  GSL_SET_COMPLEX (z, ar + br, ai + bi);
  return z;
}

gsl_complex
gsl_complex_add_real (gsl_complex a, float x)
{                               /* z=a+x */
  gsl_complex z;
  GSL_SET_COMPLEX (z, GSL_REAL (a) + x, GSL_IMAG (a));
  return z;
}

gsl_complex
gsl_complex_add_imag (gsl_complex a, float y)
{                               /* z=a+iy */
  gsl_complex z;
  GSL_SET_COMPLEX (z, GSL_REAL (a), GSL_IMAG (a) + y);
  return z;
}


gsl_complex
gsl_complex_sub (gsl_complex a, gsl_complex b)
{                               /* z=a-b */
  float ar = GSL_REAL (a), ai = GSL_IMAG (a);
  float br = GSL_REAL (b), bi = GSL_IMAG (b);

  gsl_complex z;
  GSL_SET_COMPLEX (z, ar - br, ai - bi);
  return z;
}

gsl_complex
gsl_complex_sub_real (gsl_complex a, float x)
{                               /* z=a-x */
  gsl_complex z;
  GSL_SET_COMPLEX (z, GSL_REAL (a) - x, GSL_IMAG (a));
  return z;
}

gsl_complex
gsl_complex_sub_imag (gsl_complex a, float y)
{                               /* z=a-iy */
  gsl_complex z;
  GSL_SET_COMPLEX (z, GSL_REAL (a), GSL_IMAG (a) - y);
  return z;
}

gsl_complex
gsl_complex_mul (gsl_complex a, gsl_complex b)
{                               /* z=a*b */
  float ar = GSL_REAL (a), ai = GSL_IMAG (a);
  float br = GSL_REAL (b), bi = GSL_IMAG (b);

  gsl_complex z;
  GSL_SET_COMPLEX (z, ar * br - ai * bi, ar * bi + ai * br);
  return z;
}

gsl_complex
gsl_complex_mul_real (gsl_complex a, float x)
{                               /* z=a*x */
  gsl_complex z;
  GSL_SET_COMPLEX (z, x * GSL_REAL (a), x * GSL_IMAG (a));
  return z;
}

gsl_complex
gsl_complex_mul_imag (gsl_complex a, float y)
{                               /* z=a*iy */
  gsl_complex z;
  GSL_SET_COMPLEX (z, -y * GSL_IMAG (a), y * GSL_REAL (a));
  return z;
}

gsl_complex
gsl_complex_div (gsl_complex a, gsl_complex b)
{                               /* z=a/b */
  float ar = GSL_REAL (a), ai = GSL_IMAG (a);
  float br = GSL_REAL (b), bi = GSL_IMAG (b);

  float s = 1.0 / gsl_complex_abs (b);

  float sbr = s * br;
  float sbi = s * bi;

  float zr = (ar * sbr + ai * sbi) * s;
  float zi = (ai * sbr - ar * sbi) * s;

  gsl_complex z;
  GSL_SET_COMPLEX (z, zr, zi);
  return z;
}

gsl_complex
gsl_complex_div_real (gsl_complex a, float x)
{                               /* z=a/x */
  gsl_complex z;
  GSL_SET_COMPLEX (z, GSL_REAL (a) / x, GSL_IMAG (a) / x);
  return z;
}

gsl_complex
gsl_complex_div_imag (gsl_complex a, float y)
{                               /* z=a/(iy) */
  gsl_complex z;
  GSL_SET_COMPLEX (z, GSL_IMAG (a) / y,  - GSL_REAL (a) / y);
  return z;
}

gsl_complex
gsl_complex_conjugate (gsl_complex a)
{                               /* z=conj(a) */
  gsl_complex z;
  GSL_SET_COMPLEX (z, GSL_REAL (a), -GSL_IMAG (a));
  return z;
}

gsl_complex
gsl_complex_negative (gsl_complex a)
{                               /* z=-a */
  gsl_complex z;
  GSL_SET_COMPLEX (z, -GSL_REAL (a), -GSL_IMAG (a));
  return z;
}

gsl_complex
gsl_complex_inverse (gsl_complex a)
{                               /* z=1/a */
  float s = 1.0 / gsl_complex_abs (a);

  gsl_complex z;
  GSL_SET_COMPLEX (z, (GSL_REAL (a) * s) * s, -(GSL_IMAG (a) * s) * s);
  return z;
}

/**********************************************************************
 * Elementary complex functions
 **********************************************************************/

gsl_complex
gsl_complex_sqrt (gsl_complex a)
{                               /* z=sqrt(a) */
  gsl_complex z;

  if (GSL_REAL (a) == 0.0 && GSL_IMAG (a) == 0.0)
    {
      GSL_SET_COMPLEX (z, 0.0, 0.0);
    }
  else
    {
      float x = abs  (GSL_REAL (a));
      float y = abs  (GSL_IMAG (a));
      float w;

      if (x >= y)
        {
          float t = y / x;
          w = sqrt (x) * sqrt (0.5 * (1.0 + sqrt (1.0 + t * t)));
        }
      else
        {
          float t = x / y;
          w = sqrt (y) * sqrt (0.5 * (t + sqrt (1.0 + t * t)));
        }

      if (GSL_REAL (a) >= 0.0)
        {
          float ai = GSL_IMAG (a);
          GSL_SET_COMPLEX (z, w, ai / (2.0 * w));
        }
      else
        {
          float ai = GSL_IMAG (a);
          float vi = (ai >= 0.0) ? w : -w;
          GSL_SET_COMPLEX (z, ai / (2.0 * vi), vi);
        }
    }

  return z;
}

gsl_complex
gsl_complex_sqrt_real (float x)
{                               /* z=sqrt(x) */
  gsl_complex z;

  if (x >= 0.0)
    {
      GSL_SET_COMPLEX (z, sqrt (x), 0.0);
    }
  else
    {
      GSL_SET_COMPLEX (z, 0.0, sqrt (-x));
    }

  return z;
}

gsl_complex
gsl_complex_exp (gsl_complex a)
{                               /* z=exp(a) */
  float rho = exp (GSL_REAL (a));
  float theta = GSL_IMAG (a);

  gsl_complex z;
  GSL_SET_COMPLEX (z, rho * cos (theta), rho * sin (theta));
  return z;
}

gsl_complex
gsl_complex_pow (gsl_complex a, gsl_complex b)
{                               /* z=a^b */
  gsl_complex z;

  if (GSL_REAL (a) == 0.0 && GSL_IMAG (a) == 0.0)
    {
      if (GSL_REAL (b) == 0.0 && GSL_IMAG (b) == 0.0)
        {
          GSL_SET_COMPLEX (z, 1.0, 0.0);
        }
      else 
        {
          GSL_SET_COMPLEX (z, 0.0, 0.0);
        }
    }
  else if (GSL_REAL (b) == 1.0 && GSL_IMAG (b) == 0.0) 
    {
      return a;
    }
  else if (GSL_REAL (b) == -1.0 && GSL_IMAG (b) == 0.0) 
    {
      return gsl_complex_inverse (a);
    }
  else
    {
      float logr = gsl_complex_logabs (a);
      float theta = gsl_complex_arg (a);

      float br = GSL_REAL (b), bi = GSL_IMAG (b);

      float rho = exp (logr * br - bi * theta);
      float beta = theta * br + bi * logr;

      GSL_SET_COMPLEX (z, rho * cos (beta), rho * sin (beta));
    }

  return z;
}

gsl_complex
gsl_complex_pow_real (gsl_complex a, float b)
{                               /* z=a^b */
  gsl_complex z;

  if (GSL_REAL (a) == 0.0 && GSL_IMAG (a) == 0.0)
    {
      if (b == 0.0)
        {
          GSL_SET_COMPLEX (z, 1.0, 0.0);
        }
      else
        {
          GSL_SET_COMPLEX (z, 0.0, 0.0);
        }
    }
  else
    {
      float logr = gsl_complex_logabs (a);
      float theta = gsl_complex_arg (a);
      float rho = exp (logr * b);
      float beta = theta * b;
      GSL_SET_COMPLEX (z, rho * cos (beta), rho * sin (beta));
    }

  return z;
}

gsl_complex
gsl_complex_log (gsl_complex a)
{                               /* z=log(a) */
  float logr = gsl_complex_logabs (a);
  float theta = gsl_complex_arg (a);

  gsl_complex z;
  GSL_SET_COMPLEX (z, logr, theta);
  return z;
}

gsl_complex
gsl_complex_log10 (gsl_complex a)
{                               /* z = log10(a) */
  return gsl_complex_mul_real (gsl_complex_log (a), 1.0 / log (10.));
}

gsl_complex
gsl_complex_log_b (gsl_complex a, gsl_complex b)
{
  return gsl_complex_div (gsl_complex_log (a), gsl_complex_log (b));
}

/***********************************************************************
 * Complex trigonometric functions
 ***********************************************************************/

gsl_complex
gsl_complex_sin (gsl_complex a)
{                               /* z = sin(a) */
  float R = GSL_REAL (a), I = GSL_IMAG (a);

  gsl_complex z;

  if (I == 0.0) 
    {
      /* avoid returing negative zero (-0.0) for the imaginary part  */

      GSL_SET_COMPLEX (z, sin (R), 0.0);  
    } 
  else 
    {
      GSL_SET_COMPLEX (z, sin (R) * cosh (I), cos (R) * sinh (I));
    }

  return z;
}

gsl_complex
gsl_complex_cos (gsl_complex a)
{                               /* z = cos(a) */
  float R = GSL_REAL (a), I = GSL_IMAG (a);

  gsl_complex z;

  if (I == 0.0) 
    {
      /* avoid returing negative zero (-0.0) for the imaginary part  */

      GSL_SET_COMPLEX (z, cos (R), 0.0);  
    } 
  else 
    {
      GSL_SET_COMPLEX (z, cos (R) * cosh (I), sin (R) * sinh (-I));
    }

  return z;
}

gsl_complex
gsl_complex_tan (gsl_complex a)
{                               /* z = tan(a) */
  float R = GSL_REAL (a), I = GSL_IMAG (a);

  gsl_complex z;

  if (abs  (I) < 1.0)
    {
      float D = pow (cos (R), 2.0) + pow (sinh (I), 2.0);

      GSL_SET_COMPLEX (z, 0.5 * sin (2.0 * R) / D, 0.5 * sinh (2.0 * I) / D);
    }
  else
    {
      float D = pow (cos (R), 2.0) + pow (sinh (I), 2.0);
      float F = 1.0 + pow(cos (R)/sinh (I), 2.0);

      GSL_SET_COMPLEX (z, 0.5 * sin (2.0 * R) / D, 1.0 / (tanh (I) * F));
    }

  return z;
}

gsl_complex
gsl_complex_sec (gsl_complex a)
{                               /* z = sec(a) */
  gsl_complex z = gsl_complex_cos (a);
  return gsl_complex_inverse (z);
}

gsl_complex
gsl_complex_csc (gsl_complex a)
{                               /* z = csc(a) */
  gsl_complex z = gsl_complex_sin (a);
  return gsl_complex_inverse(z);
}


gsl_complex
gsl_complex_cot (gsl_complex a)
{                               /* z = cot(a) */
  gsl_complex z = gsl_complex_tan (a);
  return gsl_complex_inverse (z);
}

/**********************************************************************
 * Inverse Complex Trigonometric Functions
 **********************************************************************/

gsl_complex
gsl_complex_arcsin_real (float a)
{                               /* z = arcsin(a) */
  gsl_complex z;

  if (abs  (a) <= 1.0)
    {
      GSL_SET_COMPLEX (z, asin (a), 0.0);
    }
  else
    {
      if (a < 0.0)
        {
          GSL_SET_COMPLEX (z, -M_PI_2, acosh (-a));
        }
      else
        {
          GSL_SET_COMPLEX (z, M_PI_2, -acosh (a));
        }
    }

  return z;
}


gsl_complex
gsl_complex_arcsin (gsl_complex a)
{                               /* z = arcsin(a) */
  float R = GSL_REAL (a), I = GSL_IMAG (a);
  gsl_complex z;

  if (I == 0.0)
    {
      z = gsl_complex_arcsin_real (R);
    }
  else
    {
      float x = abs  (R), y = abs  (I);
      float r = gsl_hypot (x + 1.0, y), s = gsl_hypot (x - 1.0, y);
      float A = 0.5 * (r + s);
      float B = x / A;
      float y2 = y * y;

      float real, imag;

      const float A_crossover = 1.5, B_crossover = 0.6417;

      if (B <= B_crossover)
        {
          real = asin (B);
        }
      else
        {
          if (x <= 1.0)
            {
              float D = 0.5 * (A + x) * (y2 / (r + x + 1.0) + (s + (1.0 - x)));
              real = atan (x / sqrt (D));
            }
          else
            {
              float Apx = A + x;
              float D = 0.5 * (Apx / (r + x + 1.0) + Apx / (s + (x - 1.0)));
              real = atan (x / (y * sqrt (D)));
            }
        }

      if (A <= A_crossover)
        {
          float Am1;

          if (x < 1.0)
            {
              Am1 = 0.5 * (y2 / (r + (x + 1.0)) + y2 / (s + (1.0 - x)));
            }
          else
            {
              Am1 = 0.5 * (y2 / (r + (x + 1.0)) + (s + (x - 1.0)));
            }

          imag = log (Am1 + sqrt (Am1 * (A + 1.0)));
        }
      else
        {
          imag = log (A + sqrt (A * A - 1.0));
        }

      GSL_SET_COMPLEX (z, (R >= 0.0) ? real : -real, (I >= 0.0) ? imag : -imag);
    }

  return z;
}

gsl_complex
gsl_complex_arccos_real (float a)
{                               /* z = arccos(a) */
  gsl_complex z;

  if (abs  (a) <= 1.0)
    {
      GSL_SET_COMPLEX (z, acos (a), 0.0);
    }
  else
    {
      if (a < 0.0)
        {
          GSL_SET_COMPLEX (z, M_PI, -acosh (-a));
        }
      else
        {
          GSL_SET_COMPLEX (z, 0.0, acosh (a));
        }
    }

  return z;
}

gsl_complex
gsl_complex_arccos (gsl_complex a)
{                               /* z = arccos(a) */
  float R = GSL_REAL (a), I = GSL_IMAG (a);
  gsl_complex z;

  if (I == 0.0)
    {
      z = gsl_complex_arccos_real (R);
    }
  else
    {
      float x = abs  (R), y = abs  (I);
      float r = gsl_hypot (x + 1.0, y), s = gsl_hypot (x - 1.0, y);
      float A = 0.5 * (r + s);
      float B = x / A;
      float y2 = y * y;

      float real, imag;

      const float A_crossover = 1.5, B_crossover = 0.6417;

      if (B <= B_crossover)
        {
          real = acos (B);
        }
      else
        {
          if (x <= 1.0)
            {
              float D = 0.5 * (A + x) * (y2 / (r + x + 1.0) + (s + (1.0 - x)));
              real = atan (sqrt (D) / x);
            }
          else
            {
              float Apx = A + x;
              float D = 0.5 * (Apx / (r + x + 1.0) + Apx / (s + (x - 1.0)));
              real = atan ((y * sqrt (D)) / x);
            }
        }

      if (A <= A_crossover)
        {
          float Am1;

          if (x < 1.0)
            {
              Am1 = 0.5 * (y2 / (r + (x + 1.0)) + y2 / (s + (1.0 - x)));
            }
          else
            {
              Am1 = 0.5 * (y2 / (r + (x + 1.0)) + (s + (x - 1.0)));
            }

          imag = log (Am1 + sqrt (Am1 * (A + 1.0)));
        }
      else
        {
          imag = log (A + sqrt (A * A - 1.0));
        }

      GSL_SET_COMPLEX (z, (R >= 0.0) ? real : M_PI - real, (I >= 0.0) ? -imag : imag);
    }

  return z;
}

gsl_complex
gsl_complex_arctan (gsl_complex a)
{                               /* z = arctan(a) */
  float R = GSL_REAL (a), I = GSL_IMAG (a);
  gsl_complex z;

  if (I == 0.0)
    {
      GSL_SET_COMPLEX (z, atan (R), 0.0);
    }
  else
    {
      /* FIXME: This is a naive implementation which does not fully
         take into account cancellation errors, overflow, underflow
         etc.  It would benefit from the Hull et al treatment. */

      float r = gsl_hypot (R, I);

      float imag;

      float u = 2.0 * I / (1.0 + r * r);

      /* FIXME: the following cross-over should be optimized but 0.1
         seems to work ok */

      if (abs  (u) < 0.1)
        {
          imag = 0.25 * (log (u) - log (-u));
        }
      else
        {
          float A = gsl_hypot (R, I + 1.0);
          float B = gsl_hypot (R, I - 1.0);
          imag = 0.5 * log (A / B);
        }

      if (R == 0.0)
        {
          if (I > 1.0)
            {
              GSL_SET_COMPLEX (z, M_PI_2, imag);
            }
          else if (I < -1.0)
            {
              GSL_SET_COMPLEX (z, -M_PI_2, imag);
            }
          else
            {
              GSL_SET_COMPLEX (z, 0.0, imag);
            };
        }
      else
        {
          GSL_SET_COMPLEX (z, 0.5 * atan ((2.0 * R) / ((1.0 + r) * (1.0 - r))), imag);
        }
    }

  return z;
}

gsl_complex
gsl_complex_arcsec (gsl_complex a)
{                               /* z = arcsec(a) */
  gsl_complex z = gsl_complex_inverse (a);
  return gsl_complex_arccos (z);
}

gsl_complex
gsl_complex_arcsec_real (float a)
{                               /* z = arcsec(a) */
  gsl_complex z;

  if (a <= -1.0 || a >= 1.0)
    {
      GSL_SET_COMPLEX (z, acos (1.0 / a), 0.0);
    }
  else
    {
      if (a >= 0.0)
        {
          GSL_SET_COMPLEX (z, 0.0, acosh (1.0 / a));
        }
      else
        {
          GSL_SET_COMPLEX (z, M_PI, -acosh (-1.0 / a));
        }
    }

  return z;
}

gsl_complex
gsl_complex_arccsc (gsl_complex a)
{                               /* z = arccsc(a) */
  gsl_complex z = gsl_complex_inverse (a);
  return gsl_complex_arcsin (z);
}

gsl_complex
gsl_complex_arccsc_real (float a)
{                               /* z = arccsc(a) */
  gsl_complex z;

  if (a <= -1.0 || a >= 1.0)
    {
      GSL_SET_COMPLEX (z, asin (1.0 / a), 0.0);
    }
  else
    {
      if (a >= 0.0)
        {
          GSL_SET_COMPLEX (z, M_PI_2, -acosh (1.0 / a));
        }
      else
        {
          GSL_SET_COMPLEX (z, -M_PI_2, acosh (-1.0 / a));
        }
    }

  return z;
}

gsl_complex
gsl_complex_arccot (gsl_complex a)
{                               /* z = arccot(a) */
  gsl_complex z;

  if (GSL_REAL (a) == 0.0 && GSL_IMAG (a) == 0.0)
    {
      GSL_SET_COMPLEX (z, M_PI_2, 0.0);
    }
  else
    {
      z = gsl_complex_inverse (a);
      z = gsl_complex_arctan (z);
    }

  return z;
}

/**********************************************************************
 * Complex Hyperbolic Functions
 **********************************************************************/

gsl_complex
gsl_complex_sinh (gsl_complex a)
{                               /* z = sinh(a) */
  float R = GSL_REAL (a), I = GSL_IMAG (a);

  gsl_complex z;
  GSL_SET_COMPLEX (z, sinh (R) * cos (I), cosh (R) * sin (I));
  return z;
}

gsl_complex
gsl_complex_cosh (gsl_complex a)
{                               /* z = cosh(a) */
  float R = GSL_REAL (a), I = GSL_IMAG (a);

  gsl_complex z;
  GSL_SET_COMPLEX (z, cosh (R) * cos (I), sinh (R) * sin (I));
  return z;
}

gsl_complex
gsl_complex_tanh (gsl_complex a)
{                               /* z = tanh(a) */
  float R = GSL_REAL (a), I = GSL_IMAG (a);

  gsl_complex z;

  if (abs (R) < 1.0) 
    {
      float D = pow (cos (I), 2.0) + pow (sinh (R), 2.0);
      
      GSL_SET_COMPLEX (z, sinh (R) * cosh (R) / D, 0.5 * sin (2.0 * I) / D);
    }
  else
    {
      float D = pow (cos (I), 2.0) + pow (sinh (R), 2.0);
      float F = 1.0 + pow (cos (I) / sinh (R), 2.0);

      GSL_SET_COMPLEX (z, 1.0 / (tanh (R) * F), 0.5 * sin (2.0 * I) / D);
    }

  return z;
}

gsl_complex
gsl_complex_sech (gsl_complex a)
{                               /* z = sech(a) */
  gsl_complex z = gsl_complex_cosh (a);
  return gsl_complex_inverse (z);
}

gsl_complex
gsl_complex_csch (gsl_complex a)
{                               /* z = csch(a) */
  gsl_complex z = gsl_complex_sinh (a);
  return gsl_complex_inverse (z);
}

gsl_complex
gsl_complex_coth (gsl_complex a)
{                               /* z = coth(a) */
  gsl_complex z = gsl_complex_tanh (a);
  return gsl_complex_inverse (z);
}

/**********************************************************************
 * Inverse Complex Hyperbolic Functions
 **********************************************************************/

gsl_complex
gsl_complex_arcsinh (gsl_complex a)
{                               /* z = arcsinh(a) */
  gsl_complex z = gsl_complex_mul_imag(a, 1.0);
  z = gsl_complex_arcsin (z);
  z = gsl_complex_mul_imag (z, -1.0);
  return z;
}

gsl_complex
gsl_complex_arccosh (gsl_complex a)
{                               /* z = arccosh(a) */
  gsl_complex z = gsl_complex_arccos (a);
  z = gsl_complex_mul_imag (z, GSL_IMAG(z) > 0.0 ? -1.0 : 1.0);
  return z;
}

gsl_complex
gsl_complex_arccosh_real (float a)
{                               /* z = arccosh(a) */
  gsl_complex z;

  if (a >= 1.0)
    {
      GSL_SET_COMPLEX (z, acosh (a), 0.0);
    }
  else
    {
      if (a >= -1.0)
        {
          GSL_SET_COMPLEX (z, 0.0, acos (a));
        }
      else
        {
          GSL_SET_COMPLEX (z, acosh (-a), M_PI);
        }
    }

  return z;
}

gsl_complex
gsl_complex_arctanh_real (float a)
{                               /* z = arctanh(a) */
  gsl_complex z;

  if (a > -1.0 && a < 1.0)
    {
      GSL_SET_COMPLEX (z, atanh (a), 0.0);
    }
  else
    {
      GSL_SET_COMPLEX (z, atanh (1.0 / a), (a < 0.0) ? M_PI_2 : -M_PI_2);
    }

  return z;
}

gsl_complex
gsl_complex_arctanh (gsl_complex a)
{                               /* z = arctanh(a) */
  if (GSL_IMAG (a) == 0.0)
    {
      return gsl_complex_arctanh_real (GSL_REAL (a));
    }
  else
    {
      gsl_complex z = gsl_complex_mul_imag(a, 1.0);
      z = gsl_complex_arctan (z);
      z = gsl_complex_mul_imag (z, -1.0);
      return z;
    }
}

gsl_complex
gsl_complex_arcsech (gsl_complex a)
{                               /* z = arcsech(a); */
  gsl_complex t = gsl_complex_inverse (a);
  return gsl_complex_arccosh (t);
}

gsl_complex
gsl_complex_arccsch (gsl_complex a)
{                               /* z = arccsch(a) */
  gsl_complex t = gsl_complex_inverse (a);
  return gsl_complex_arcsinh (t);
}

gsl_complex
gsl_complex_arccoth (gsl_complex a)
{                               /* z = arccoth(a) */
  gsl_complex t = gsl_complex_inverse (a);
  return gsl_complex_arctanh (t);
}

// Math utils functions =================================
vec3 rotate3dY(vec3 v, float a) {
    float cosA = cos(a);
    float sinA = sin(a);
    return vec3(
        v.x * cosA + v.z * sinA,
        v.y,
        -v.x * sinA + v.z * cosA
    );
}

vec3 rotate3dX(vec3 v, float a) {
    float cosA = cos(a);
    float sinA = sin(a);
    return vec3(
        v.x,
        v.y * cosA - v.z * sinA,
        v.y * sinA + v.z * cosA
    );
}

float checker(vec3 p) {
  float t = 10.0;
  return step(0.0, sin(PI * p.x + PI/t)*sin(PI *p.y + PI/t)*sin(PI *p.z + PI/t));
}

// Hash implementation
uint hash(ivec3 val) {
  return uint(val.x + WORLD_SIZE.y * (val.y + WORLD_SIZE.z * val.z));
}

// Intersections code
// Thanks to iq's https://www.shadertoy.com/view/XtlBDs
// 0--b--3
// |\
// a c
// |  \
// 1    2
//
vec3 quadIntersect( in vec3 ro, in vec3 rd, in vec3 v0, in vec3 v1, in vec3 v2, in vec3 v3 ) {
    // lets make v0 the origin
    vec3 a = v1 - v0;
    vec3 b = v3 - v0;
    vec3 c = v2 - v0;
    vec3 p = ro - v0;

    // intersect plane
    vec3 nor = cross(a,b);
    float t = -dot(p,nor)/dot(rd,nor);
    if( t<0.0 ) return vec3(-1.0);

    // intersection point
    return p + t*rd;
}

float computeDistance(vec3 A, vec3 B, vec3 C) {
	float x = length(cross(B - A, C - A));
	float y = length(B - A);
	return x / y;
}

// Ray marching code
void step_ray(inout RayObject ray) {
  ray.mask = lessThanEqual(ray.side_dist.xyz, min(ray.side_dist.yzx, ray.side_dist.zxy));
  ray.side_dist += vec3(ray.mask) * ray.delta_dist;
  ray.map_pos += ivec3(vec3(ray.mask)) * ray.step;
}

// the ray will simply iterate over the space in the direction it's facing trying to hit a 'solid' object
// Walls: will stop
// Models: will stop
// Mirrors: will bounce off the mirror and continue iterating
// Light source: will stop
// Optical Object: will multiply it's internal jones matrix and continue iterating
void iterateRayInDirection(inout RayObject ray) {
  for (int i = 0; i < MAX_RAY_STEPS; i += 1) {
    uint hashed_value = hash(ray.map_pos + ivec3(100, 100, 100));
    uint original_index = hashed_value % uint(1000);
    uint current_index = buckets[original_index];

    vec3 quad_hit_position = quadIntersect(
      ray.pos, 
      ray.dir, 
      vec3(49.9, 49.4, 51.2), 
      vec3(49.5, 49.2, 51.2), 
      vec3(49.5, 49.2, 51.8), 
      vec3(49.5, 49.8, 51.8)
    );

    // TEMPORAL check if it hits a random plane
    if (ray.map_pos == ivec3(49, 49, 51) &&
      (
        length(quad_hit_position) < 0.2
      )) {
      // ray.distance_traveled = length(vec3(ray.mask) * (ray.side_dist - ray.delta_dist));
      // ray.current_real_position = vec3(50.5, 50.8, 50.2) + quad_hit_position;
      // ray.distance_traveled = length(ray.current_real_position - ray.pos);

      // ray.distance_traveled += length((quad_hit_position + quad_center) - ray.current_real_position);
      // ray.current_real_position = ray.pos + ray.dir * ray.distance_traveled;

      ray.object_hit = uint(0);

      float h = 2.0 + checker(quad_hit_position * 10.0);
      ray.color *= vec4(h, h, h, 1);
      ray.object_hit = uint(1);

      ray.ended_in_hit = true;
      return;
    }

    while (objects[(current_index * uint(3)) + uint(2)] != U32_MAX) {
      if (objects[current_index * uint(3)] == hashed_value) {
        ray.ended_in_hit = true;
        ray.object_hit = current_index + uint(1);
        return;
      }

      current_index = objects[(current_index * uint(3)) + uint(2)];
    }

    if (objects[current_index * uint(3)] == hashed_value) {
      ray.ended_in_hit = true;
      ray.object_hit = objects[current_index * uint(3) + uint(1)]; // key.val
      return;
    }

    if ((ray.map_pos.x > 100 || ray.map_pos.x <= 0) || 
        (ray.map_pos.y > 100 || ray.map_pos.y <= 0) ||
        (ray.map_pos.z > 100 || ray.map_pos.z <= 0)
    ) {
      ray.distance_traveled = length(vec3(ray.mask) * (ray.side_dist - ray.delta_dist));
      ray.current_real_position = ray.pos + ray.dir * ray.distance_traveled;
      ray.object_hit = uint(0);

      float h = 2.0 + checker(ray.current_real_position);
      ray.color *= vec4(h, h, h, 1);

      ray.ended_in_hit = true;
      return;
    }

    step_ray(ray);
  }
}

void main() {
  vec2 viewport_dimensions = vec2(750., 750.);
  vec2 screen_pos = ((gl_FragCoord.xy / viewport_dimensions) * 2.) - 1.;

  vec3 camera_dir = vec3(0.0, 0.0, 1.0);
  vec3 camera_plane_u = vec3(1.0, 0.0, 0.0);
  vec3 camera_plane_v = vec3(0.0, 1.0, 0.0);

  vec3 ray_dir = camera_dir + screen_pos.x * camera_plane_u + screen_pos.y * camera_plane_v;
  ray_dir = rotate3dX(ray_dir, u_rotation.y);
  ray_dir = rotate3dY(ray_dir, u_rotation.x);
  ray_dir = normalize(ray_dir);

  RayObject ray;
    ray.dir = ray_dir;
    ray.pos = position;
    ray.map_pos = ivec3(ray.pos);
    ray.delta_dist = 1.0 / abs(ray.dir);
    ray.step = ivec3(sign(ray.dir));
    ray.side_dist = (sign(ray.dir) * (vec3(ray.map_pos) - ray.pos) + (sign(ray.dir) * 0.5) + 0.5) * ray.delta_dist;
    ray.mask = lessThanEqual(ray.side_dist.xyz, min(ray.side_dist.yzx, ray.side_dist.zxy));
    ray.color = vec4(1.0);
    ray.distance_traveled = length(vec3(ray.mask) * (ray.side_dist - ray.delta_dist));
    ray.current_real_position = ray.pos + ray.dir * length(vec3(ray.mask) * (ray.side_dist - ray.delta_dist));
    ray.ended_in_hit = false;
    // ray.optical_objects_through_which_it_passed = 0;

  iterateRayInDirection(ray);

  object_found = vec4(float(ray.object_hit) / 255.0, 0.0, 0.0, 0.0);

  if (ray.ended_in_hit) {
    out_color = vec4(vec3(ray.mask) * 0.2, 1.0) * ray.color;
  } else {
    out_color = vec4(ray.color.x, ray_dir.y, ray_dir.z, 1.);
  }
}
