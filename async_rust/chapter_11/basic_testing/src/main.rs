


pub trait AsyncProcess<X, Y, Z> {

    fn spawn(&self, input: X) -> Result<Y, String>;

    fn get_result(&self, key: Y) -> Result<Z, String>;

}


fn do_something<T>(async_handle: T, input: i32) -> Result<i32, String> 
    where T: AsyncProcess<i32, String, i32>
{
    let key = async_handle.spawn(input)?;
    println!("something is happening");
    let result = async_handle.get_result(key)?;
    if result > 10 {
        return Err("result is too big".to_string());
    }
    if result == 8 {
        return Ok(result * 2)
    }
    Ok(result * 3)
}



fn main() {
    println!("Hello, world!");
}


#[cfg(test)]
mod get_team_processes_tests {

    use super::*;
    use mockall::predicate::*;
    use mockall::mock;


    mock! {
        DatabaseHandler {}
        impl AsyncProcess<i32, String, i32> for DatabaseHandler {
            fn spawn(&self, input: i32) -> Result<String, String>;

            fn get_result(&self, key: String) -> Result<i32, String>;
        }
    }

    #[test]
    fn do_something_fail() {

        // Arrange
        let mut handle = MockDatabaseHandler::new();

        handle.expect_spawn()
                 .with(eq(4))
                 .returning(|_|{Ok("test_key".to_string())});

        handle.expect_get_result()
                 .with(eq("test_key".to_string()))
                 .returning(|_|{Ok(11)});

        // Act
        let outcome = do_something(handle, 4);

        // Assert
        assert_eq!(outcome, Err("result is too big".to_string()));
    }

}