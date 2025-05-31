[Perro Gato en Python]

El PerroGato es un programa de Python similar a fizz buzz, pero con un giro. En lugar de imprimir números, imprime "
Perro"
para los múltiplos de 3, "Gato" para los múltiplos de 5 y "PerroGato" para los múltiplos de ambos.
Si el número no es múltiplo de ninguno, simplemente imprime el número.

ejemplo:

```python
for i in range(1, 101):
    if i % 3 == 0 and i % 5 == 0:
        print("PerroGato")
    elif i % 3 == 0:
        print("Perro")
    elif i % 5 == 0:
        print("Gato")
    else:
        print(i)
```
