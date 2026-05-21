using GregModmanager.Services;
using Xunit;

namespace GregModmanager.Tests;

public class HeadlessRunnerTests
{
    [Theory]
    [InlineData("--help")]
    [InlineData("-h")]
    public void TryHandle_WithHelpArg_ReturnsTrueAndExitCodeZero(string helpArg)
    {
        // Arrange
        var args = new[] { helpArg };

        // Act
        var result = HeadlessRunner.TryHandle(args, out var exitCode);

        // Assert
        Assert.True(result);
        Assert.Equal(0, exitCode);
    }

    [Theory]
    [InlineData("some_other_arg")]
    [InlineData("--mode", "something_else")]
    [InlineData("")]
    public void TryHandle_NotPublishInvocation_ReturnsFalse(params string[] args)
    {
        // Act
        var result = HeadlessRunner.TryHandle(args, out var exitCode);

        // Assert
        Assert.False(result);
        Assert.Equal(0, exitCode);
    }

    [Theory]
    [InlineData("--upload")]
    [InlineData("--mode", "publish")]
    public void TryHandle_PublishInvocationWithoutPath_ReturnsTrueAndExitCodeTwo(params string[] args)
    {
        // Act
        var result = HeadlessRunner.TryHandle(args, out var exitCode);

        // Assert
        Assert.True(result);
        Assert.Equal(2, exitCode);
    }
}
