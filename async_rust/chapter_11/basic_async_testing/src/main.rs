use async_trait::async_trait;


#[async_trait]
pub trait AsyncProcess<X, Z> {

    async fn get_result(&self, key: X) -> Result<Z, String>;

}


async fn do_something<T>(async_handle: T, input: i32) -> Result<i32, String> 
    where T: AsyncProcess<i32, i32> + Send + Sync + 'static
{
    let future = tokio::task::spawn(async move {
        async_handle.get_result(input).await
    });
    println!("something is happening");
    let result: i32 = future.await.unwrap()?;
    if result > 10 {
        return Err("result is too big".to_string());
    }
    if result == 8 {
        return Ok(result * 2)
    }
    Ok(result * 3)
}


// Pin<Box<dyn Future<Output = T> + Send + 'static>>



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
        #[async_trait]
        impl AsyncProcess<i32, i32> for DatabaseHandler {
            async fn get_result(&self, key: i32) -> Result<i32, String>;
        }
    }

    #[test]
    fn do_something_fail() {
        let mut handle = MockDatabaseHandler::new();

        handle.expect_get_result()
                 .with(eq(4))
                 .returning(|_|{Ok(11)});

        let runtime = tokio::runtime::Builder::new_current_thread().enable_all()
                                                                   .build()
                                                                   .unwrap();
        let outcome = runtime.block_on(do_something(handle, 4));
        assert_eq!(outcome, Err("result is too big".to_string()));
    }

}