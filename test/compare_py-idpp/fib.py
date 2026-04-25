def fib(n):
    if n < 2:
        return n
    return fib(n - 1) + fib(n - 2)

angka = 25
print(f"Menghitung Fibonacci ke-{angka}...")
hasil = fib(angka)
print(f"Hasil: {hasil}")