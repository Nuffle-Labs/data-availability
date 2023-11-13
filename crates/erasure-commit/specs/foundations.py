# Taken from https://eprint.iacr.org/2023/1079.pdf
import math

# Some constants .
# Sizes of group elements , field elements , and hashes in bits
BLS_FE_SIZE = 48.0 * 8.0
BLS_GE_SIZE = 48.0 * 8.0
# Let ’s say we use the SECP256_k1 curve
PEDERSEN_FE_SIZE = 32.0 * 8.0
PEDERSEN_GE_SIZE = 33.0 * 8.0
# Let ’s say we use SHA256
HASH_SIZE = 256
# Statistical Security Parameter for Soundness
SECPAR = 40


# ###############################################################
# Structure of a code :
# Constructor takes as input parameters (e.g. fieldsize , inputdatalength , rate , ..)
# sizemessagesymbol : size of a symbol in the message alphabet
# messagelength : number of symbols of a message
# sizecodesymbol : size of a symbol in the code alphabet
# codewordlength : number of symbols of a codeword
# reception : reception efficiency


# ###############################################################
# Reed - Solomon Code
# Polynomial of degree k -1 over field with field element length fsize
# Evaluated at n points
def makeRSCode(fsize, k, n):
    code = {}
    code[" sizemessagesymbol "] = fsize
    code[" messagelength "] = k
    code[" sizecodesymbol "] = fsize
    code[" codewordlength "] = n
    code[" reception "] = k
    return code


# Interleaved Code
# Encode ell rows with basecode
# Columns of the resulting matrix
# are the symbols of codeword
def makeInterleavedCode(basecode, ell):
    code = {}
    code[" sizemessagesymbol "] = basecode[" sizemessagesymbol "]
    code[" messagelength "] = basecode[" messagelength "] * ell
    code[" sizecodesymbol "] = basecode[" sizecodesymbol "] * ell
    code[" codewordlength "] = basecode[" codewordlength "]
    code[" reception "] = basecode[" reception "]
    return code


# Tensor Code of two given codes rowcode , columncode
def makeTensorCode(rowcode, columncode):
    assert rowcode[" sizemessagesymbol "] == columncode[" sizemessagesymbol "]
    assert rowcode[" sizecodesymbol "] == columncode[" sizecodesymbol "]
    assert rowcode[" sizemessagesymbol "] == rowcode[" sizecodesymbol "]
    code = {}
    code[" sizemessagesymbol "] = rowcode[" sizemessagesymbol "]
    code[" messagelength "] = rowcode[" messagelength "] * columncode[" messagelength "]
    code[" sizecodesymbol "] = rowcode[" sizecodesymbol "]
    code[" codewordlength "] = (
        rowcode[" codewordlength "] * columncode[" codewordlength "]
    )
    rowdist = rowcode[" codewordlength "] - rowcode[" reception "] + 1
    coldist = columncode[" codewordlength "] - columncode[" reception "] + 1
    code[" reception "] = code[" codewordlength "] - rowdist * coldist + 1
    return code


# ###############################################################
# Structure of a scheme :
# Constructor takes size of data in bits and scheme specific parameters as input
# comsize : size of commitment in bits
# code : code that is used
# encodingsymbolsize : size of a symbol in the encoding in bits
# encodinglength : number of symbols of the encoding
# commpqsize : size of communication per query in bits
# reception : reception efficiency
# samples : minimum number of samples to get prob . of reconstructing below 2^{ - SECPAR }.
# Samples are computed using analytical results


# ###############################################################
# Naive scheme
# Put all the data in one symbol , and let the commitment be a hash
def makeNaiveScheme(datasize):
    scheme = {}
    scheme[" encodingsymbolsize "] = datasize
    scheme[" comsize "] = HASH_SIZE
    scheme[" encodinglength "] = 1
    scheme[" commpqsize "] = datasize
    scheme[" reception "] = 1
    scheme[" samples "] = 1
    return scheme


# Merkle scheme
# Take a merkle tree and the identity code
def makeMerkleScheme(datasize, chunksize=1024):
    k = math.ceil(datasize / chunksize)
    scheme = {}
    scheme[" encodingsymbolsize "] = chunksize + math.log(k, 2) * HASH_SIZE
    scheme[" comsize "] = HASH_SIZE
    scheme[" encodinglength "] = k
    scheme[" commpqsize "] = math.ceil(math.log(k, 2)) + scheme[" encodingsymbolsize "]
    scheme[" reception "] = k
    scheme[" samples "] = math.ceil(
        (k / math.log(math.e, 2)) * (math.log(k, 2) + SECPAR)
    )
    return scheme


# KZG Commitment , interpreted as an erasure code commitment for the RS code
# The RS Code is set to have parameters k,n with n = invrate * k
def makeKZGScheme(datasize, invrate=4):
    k = math.ceil(datasize / BLS_FE_SIZE)
    n = k * invrate
    rscode = makeRSCode(BLS_FE_SIZE, k, n)
    scheme = {}
    scheme[" encodingsymbolsize "] = rscode[" sizecodesymbol "] + BLS_GE_SIZE
    scheme[" comsize "] = BLS_GE_SIZE
    scheme[" encodinglength "] = rscode[" codewordlength "]
    scheme[" commpqsize "] = math.ceil(math.log(n, 2)) + scheme[" encodingsymbolsize "]
    scheme[" reception "] = rscode[" reception "]
    scheme[" samples "] = math.ceil(
        SECPAR + (1.0 - math.log(math.e, 1 / invrate)) * (k - 1)
    )
    return scheme


# Tensor Code Commitment , where each dimension is expanded with inverse rate invrate .
# That is , data is a k x k matrix , and the codeword is a n x n matrix , with n = invrate * k
# Both column and row code are RS codes .
def makeTensorScheme(datasize, invrate=4):
    numfe = math.ceil(datasize / BLS_FE_SIZE)
    k = math.ceil(math.sqrt(numfe))
    # assert (k*k == numfe )
    n = invrate * k
    columncode = makeRSCode(BLS_FE_SIZE, k, n)
    rowcode = makeRSCode(BLS_FE_SIZE, k, n)
    tensorcode = makeTensorCode(rowcode, columncode)
    scheme = {}
    scheme[" encodingsymbolsize "] = columncode[" sizecodesymbol "] + BLS_GE_SIZE
    scheme[" comsize "] = rowcode[" codewordlength "] * BLS_GE_SIZE
    scheme[" encodinglength "] = tensorcode[" codewordlength "]
    scheme[" commpqsize "] = math.ceil(math.log(n, 2)) + scheme[" encodingsymbolsize "]
    scheme[" reception "] = tensorcode[" reception "]
    r = tensorcode[" reception "] / tensorcode[" codewordlength "]
    scheme[" samples "] = math.ceil(SECPAR + (1.0 - math.log(math.e, r)) * (k * k - 1))
    return scheme


# Hash - Based Code Commitment , over field with elements of size fsize ,
# parallel repetition parameters P and L. Data is treated as a k x k matrix ,
# and codewords are k x n matrices , where n = k* invrate .
def makeHashBasedScheme(datasize, fsize=32, P=8, L=64, invrate=4):
    numfe = math.ceil(datasize / fsize)
    k = math.ceil(math.sqrt(numfe))
    n = invrate * k
    basecode = makeRSCode(fsize, k, n)
    code = makeInterleavedCode(basecode, k)
    scheme = {}
    scheme[" encodingsymbolsize "] = k * fsize
    scheme[" comsize "] = n * HASH_SIZE + P * n * fsize + L * k * fsize
    scheme[" encodinglength "] = n
    scheme[" commpqsize "] = math.ceil(math.log(n, 2)) + scheme[" encodingsymbolsize "]
    scheme[" reception "] = code[" reception "]
    r = code[" reception "] / code[" codewordlength "]
    scheme[" samples "] = math.ceil(SECPAR + (1.0 - math.log(math.e, r)) * (k - 1))
    return scheme


# Homomorphic Hash - Based Code Commitment
# instantiated with Pedersen Hash
# parallel repetition parameters P and L. Data is treated as a k x k matrix ,
# and codewords are k x n matrices , where n = k* invrate .
def makeHomHashBasedScheme(datasize, P=2, L=2, invrate=4):
    numfe = math.ceil(datasize / PEDERSEN_FE_SIZE)
    k = math.ceil(math.sqrt(numfe))
    n = invrate * k
    basecode = makeRSCode(PEDERSEN_FE_SIZE, k, n)
    code = makeInterleavedCode(basecode, k)
    scheme = {}
    scheme[" encodingsymbolsize "] = k * PEDERSEN_FE_SIZE
    scheme[" comsize "] = (
        n * PEDERSEN_GE_SIZE + P * n * PEDERSEN_FE_SIZE + L * k * PEDERSEN_FE_SIZE
    )
    scheme[" encodinglength "] = n
    scheme[" commpqsize "] = math.ceil(math.log(n, 2)) + scheme[" encodingsymbolsize "]
    scheme[" reception "] = code[" reception "]
    r = code[" reception "] / code[" codewordlength "]
    scheme[" samples "] = math.ceil(SECPAR + (1.0 - math.log(math.e, r)) * (k - 1))
    return scheme
