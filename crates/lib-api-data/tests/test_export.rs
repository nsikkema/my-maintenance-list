use lib_api_data::export;

#[test]
fn test_index_data() {
    assert_eq!(export::get_typescript_definitions(), "export type Response<T> = { data: T }\nexport type ErrorResponse<T> = { message: string; code: number | null; data: T | null }\nexport type ResponseType<T> = { Success: Response<T> } | { Fail: Response<T> } | { Error: ErrorResponse<T> }")
}
