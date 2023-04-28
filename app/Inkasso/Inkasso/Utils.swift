//
//  Utils.swift
//  Inkasso
//
//  Created by Ritam Sarmah on 3/25/23.
//

import Foundation

enum Utils {
    static func errorMessage(for response: URLResponse, with data: Data) -> String? {
        if let response = response as? HTTPURLResponse, response.statusCode != 200,
           let message = String(data: data, encoding: .utf8) {
            return "\(message) (Status: \(response.statusCode))"
        }
        
        return nil
    }
}
