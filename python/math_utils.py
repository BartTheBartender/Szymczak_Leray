def gaussian_binomial(n,k,q):

    numerator = 1
    denominator = 1

    for i in range(n-k+1, n+1):
        numerator *= (1-q**i)

    for i in range(1, k+1):
        denominator *= (1-q**i)

    return numerator//denominator

def number_of_endos():
    
    base = int(input('The characteristic of the base field: '))
    max_dim = int(input('The max dimension of the vector space: '))

    count = sum([sum([gaussian_binomial(dim,k,base) for k in range(dim)]) for dim in range(max_dim**2+1)])

    print(f'The number of endos in this category is {count}')


