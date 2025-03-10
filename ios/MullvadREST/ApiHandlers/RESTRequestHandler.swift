//
//  RESTRequestHandler.swift
//  MullvadREST
//
//  Created by pronebird on 20/04/2022.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

protocol RESTRequestHandler {
    func createURLRequest(
        endpoint: AnyIPEndpoint,
        authorization: REST.Authorization?
    ) throws -> REST.Request

    var authorizationProvider: RESTAuthorizationProvider? { get }
}

extension REST {
    struct Request {
        var urlRequest: URLRequest
        var pathTemplate: URLPathTemplate
    }

    final class AnyRequestHandler: RESTRequestHandler {
        private let _createURLRequest: (AnyIPEndpoint, REST.Authorization?) throws -> REST.Request

        let authorizationProvider: RESTAuthorizationProvider?

        init(createURLRequest: @escaping @Sendable (AnyIPEndpoint) throws -> REST.Request) {
            _createURLRequest = { endpoint, _ in
                try createURLRequest(endpoint)
            }
            authorizationProvider = nil
        }

        init(
            createURLRequest: @escaping @Sendable (AnyIPEndpoint, REST.Authorization) throws -> REST.Request,
            authorizationProvider: RESTAuthorizationProvider
        ) {
            _createURLRequest = { endpoint, authorization in
                try createURLRequest(endpoint, authorization!)
            }
            self.authorizationProvider = authorizationProvider
        }

        func createURLRequest(
            endpoint: AnyIPEndpoint,
            authorization: REST.Authorization?
        ) throws -> REST.Request {
            try _createURLRequest(endpoint, authorization)
        }
    }
}
