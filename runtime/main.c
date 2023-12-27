#include <stdio.h>
#include "values.h"
#include "print.h"
#include "runtime.h"

FILE* in;
FILE* out;

int main(int argc, char** argv)
{
  in = stdin;
  out = stdout;
  
  val_t result;

  result = entry();
  print_result(result);

  return 0;
}
