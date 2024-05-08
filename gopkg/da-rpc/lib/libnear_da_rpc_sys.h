#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>
#include <math.h>
#include <stdio.h>

#define VERSION 3

typedef struct Client Client;

typedef struct BlobSafe {
  const uint8_t *data;
  size_t len;
} BlobSafe;

typedef struct RustSafeArray {
  const uint8_t *data;
  size_t len;
} RustSafeArray;

char *get_error(void);

void clear_error(void);

/**
 * # Safety
 * null check the ptr
 */
void set_error(char *err);

/**
 * # Safety
 * we check if the pointer is null before attempting to free it
 */
void free_error(char *error);

/**
 * # Safety
 * We check if the pointers are null
 */
const struct Client *new_client_file(const char *key_path,
                                     const char *contract,
                                     const char *network,
                                     uint8_t namespace_version,
                                     uint32_t namespace_);

/**
 * # Safety
 * We check if the pointers are null
 */
const struct Client *new_client(const char *account_id,
                                const char *secret_key,
                                const char *contract,
                                const char *network,
                                uint8_t namespace_version,
                                uint32_t namespace_);

/**
 * # Safety
 * We check if the client is null
 */
void free_client(struct Client *client);

/**
 * # Safety
 * We check if the slices are null
 */
char *submit(const struct Client *client, const struct BlobSafe *blobs, size_t len);

/**
 * # Safety
 * We check if the slices are null and they should always be 32 bytes
 */
const struct BlobSafe *get(const struct Client *client, const uint8_t *transaction_id);

/**
 * # Safety
 * We check if the slices are null
 */
void free_blob(struct BlobSafe *blob);

/**
 * # Safety
 * We check if the slices are null
 */
const struct RustSafeArray *submit_batch(const struct Client *client,
                                         const char *candidate_hex,
                                         const uint8_t *tx_data,
                                         size_t tx_data_len);
