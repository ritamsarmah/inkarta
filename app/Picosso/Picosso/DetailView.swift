//
//  DetailView.swift
//  Picosso
//
//  Created by Ritam Sarmah on 3/23/23.
//

import SwiftUI

struct DetailView: View {
    
    @Environment(\.dismiss) var dismiss
    
    @ObservedObject var viewModel: DetailViewModel
    
    var body: some View {
        NavigationView {
            VStack {
                AsyncImage(url: viewModel.imageURL) { image in
                    image
                        .resizable()
                        .aspectRatio(contentMode: .fit)
                        .frame(maxHeight: 500)
                } placeholder: {
                    ProgressView()
                }
            }
            .navigationBarItems(leading: VStack(alignment: .leading, spacing: 5) {
                Text(viewModel.title)
                    .font(.system(size: 35, weight: .semibold, design: .default))
                Text(viewModel.artist)
            })
        }
    }
}

//struct DetailView_Previews: PreviewProvider {
//    static var previews: some View {
//        UploadView()
//    }
//}
