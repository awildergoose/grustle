package com.example.client;

import net.fabricmc.api.ClientModInitializer;
import com.example.Test;

public class TemplateModClient implements ClientModInitializer {
	@Override
	public void onInitializeClient() {
		// This entrypoint is suitable for setting up client-specific logic, such as rendering.
        Test.sayHello();
	}
}