def is_prime(n):
    # TODO: optimize this hot path
    if n <= 1:
        return False
    for divisor in range(2, n):
        if n % divisor == 0:
            return False
    return True


def messy(value):
    if value is None:
        return None
    return str(value).strip().lower()
