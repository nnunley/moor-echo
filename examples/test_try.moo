try
  x = 1 / 0;
except (E_DIV)
  return "Division by zero";
finally
  cleanup();
endtry